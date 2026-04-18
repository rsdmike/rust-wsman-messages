use crate::error::ApfError;
use crate::message::*;
use crate::transport::{HeciHooks, HeciTransport};

const RX_BUF_SIZE: usize = 4096;
const APF_MAX_INCOMING: usize = 512;
const INIT_ATTEMPTS: usize = 40;

pub struct ApfSession<T: HeciTransport, H: HeciHooks> {
    heci: T,
    hooks: H,
    me_addr: u8,
    host_addr: u8,
    pub(crate) sender_channel: u32,
    pub(crate) recipient_channel: u32,
    pub(crate) tx_window: u32,
    pub(crate) channel_active: bool,
    pub(crate) rx_buf: [u8; RX_BUF_SIZE],
    pub(crate) rx_len: usize,
    port_forward_ok: bool,
}

impl<T: HeciTransport, H: HeciHooks> ApfSession<T, H> {
    pub fn new(heci: T, hooks: H, me_addr: u8, host_addr: u8) -> Self {
        Self {
            heci,
            hooks,
            me_addr,
            host_addr,
            sender_channel: 0,
            recipient_channel: 0,
            tx_window: 0,
            channel_active: false,
            rx_buf: [0u8; RX_BUF_SIZE],
            rx_len: 0,
            port_forward_ok: false,
        }
    }

    pub fn port_forwarding_established(&self) -> bool { self.port_forward_ok }
    pub fn channel_active(&self) -> bool { self.channel_active }
    pub fn recipient_channel(&self) -> u32 { self.recipient_channel }
    pub fn tx_window(&self) -> u32 { self.tx_window }
    pub fn hooks_ref(&self) -> &H { &self.hooks }

    /// Test-only: bypass the handshake when unit-testing downstream operations.
    #[doc(hidden)]
    pub fn force_port_forward_ok(&mut self) {
        self.port_forward_ok = true;
    }

    /// APF init: send protocol version, consume incoming messages until
    /// the ME has forwarded port 16992 (or we time out).
    pub fn handshake(&mut self) -> Result<(), ApfError> {
        let mut out = [0u8; 93];
        encode_protocol_version(&mut out, 1, 0)?;
        self.raw_send(&out)?;

        let mut got_version = false;
        let mut timeout_count = 0u32;

        for _ in 0..INIT_ATTEMPTS {
            let mut buf = [0u8; APF_MAX_INCOMING];
            match self.heci.recv(&mut buf) {
                Ok((len, me, host)) if me == self.me_addr && host == self.host_addr => {
                    timeout_count = 0;
                    let mt = self.process_apf(&buf[..len])?;
                    if mt == APF_PROTOCOLVERSION {
                        got_version = true;
                    }
                    if self.port_forward_ok {
                        return Ok(());
                    }
                }
                Ok(_) => continue,
                Err(_) => {
                    timeout_count += 1;
                    if self.port_forward_ok {
                        return Ok(());
                    }
                    if timeout_count >= 2 {
                        return if got_version {
                            Err(ApfError::Timeout("tcpip-forward"))
                        } else {
                            Err(ApfError::Protocol("protocol version not received"))
                        };
                    }
                }
            }
        }

        if !got_version {
            return Err(ApfError::Protocol("protocol version not received"));
        }
        if !self.port_forward_ok {
            return Err(ApfError::Timeout("tcpip-forward"));
        }
        Ok(())
    }

    pub(crate) fn raw_send(&mut self, data: &[u8]) -> Result<(), ApfError> {
        self.heci.send(self.me_addr, self.host_addr, data).map_err(ApfError::from)
    }

    /// Open a `forwarded-tcpip` channel to AMT's HTTP port.
    pub fn channel_open(&mut self) -> Result<(), ApfError> {
        self.sender_channel = self.sender_channel.wrapping_add(1) % 32;
        if self.sender_channel == 0 {
            self.sender_channel = 1;
        }
        self.recipient_channel = 0;
        self.tx_window = 0;

        let mut msg = [0u8; 72];
        let n = encode_channel_open(
            &mut msg,
            self.sender_channel,
            LME_RX_WINDOW_SIZE,
            APF_AMT_HTTP_PORT,
        )?;
        self.raw_send(&msg[..n])?;
        self.hooks.post_channel_open_send();

        for _ in 0..30 {
            let mut buf = [0u8; APF_MAX_INCOMING];
            let (len, me, host) = match self.heci.recv(&mut buf) {
                Ok(v) => v,
                Err(e) => return Err(ApfError::from(e)),
            };
            if me != self.me_addr || host != self.host_addr {
                continue;
            }
            let mt = data_first_byte(&buf[..len])?;
            match mt {
                APF_CHANNEL_OPEN_CONFIRMATION => {
                    if len < 13 {
                        return Err(ApfError::Protocol("CONFIRMATION too short"));
                    }
                    self.recipient_channel = read_be32(&buf[5..9]);
                    self.tx_window = read_be32(&buf[9..13]);
                    self.channel_active = true;
                    return Ok(());
                }
                APF_CHANNEL_OPEN_FAILURE => {
                    let reason = if len >= 9 { read_be32(&buf[5..9]) } else { 0 };
                    return Err(ApfError::OpenRejected(reason));
                }
                _ => {
                    self.process_apf(&buf[..len])?;
                    continue;
                }
            }
        }
        Err(ApfError::Timeout("CHANNEL_OPEN_CONFIRMATION"))
    }

    pub fn close_channel(&mut self) {
        if self.channel_active {
            let mut msg = [0u8; 5];
            if encode_channel_close(&mut msg, self.recipient_channel).is_ok() {
                let _ = self.raw_send(&msg);
            }
            self.channel_active = false;
            self.recipient_channel = 0;
            self.tx_window = 0;
        }
    }

    pub fn close(&mut self) {
        self.close_channel();
        self.heci.close();
    }

    pub fn send_bytes(&mut self, data: &[u8]) -> Result<(), ApfError> {
        if !self.channel_active {
            return Err(ApfError::ChannelClosed);
        }
        let mut frame = [0u8; 2048];
        if 9 + data.len() > frame.len() {
            return Err(ApfError::BufferTooSmall);
        }
        let n = encode_channel_data(&mut frame, self.recipient_channel, data)?;
        self.raw_send(&frame[..n])
    }

    /// Read one or more CHANNEL_DATA frames into `out`. Returns the number
    /// of bytes copied. Stops on CHANNEL_CLOSE if any data has been
    /// received; returns `Err(Aborted)` if closed before any data arrives.
    pub fn recv_bytes(&mut self, out: &mut [u8]) -> Result<usize, ApfError> {
        self.rx_len = 0;
        for _ in 0..100 {
            let mut buf = [0u8; 2048];
            let (len, me, host) = match self.heci.recv(&mut buf) {
                Ok(v) => v,
                Err(_) if self.rx_len > 0 => break,
                Err(e) => return Err(ApfError::from(e)),
            };
            if me != self.me_addr || host != self.host_addr {
                continue;
            }
            let Some(mt) = buf.first().copied() else { continue };
            match mt {
                APF_CHANNEL_DATA => {
                    if len < 9 {
                        return Err(ApfError::Protocol("CHANNEL_DATA too short"));
                    }
                    let data_len = read_be32(&buf[5..9]) as usize;
                    if len < 9 + data_len {
                        return Err(ApfError::Protocol("CHANNEL_DATA truncated"));
                    }
                    if self.rx_len + data_len > self.rx_buf.len() {
                        return Err(ApfError::BufferTooSmall);
                    }
                    self.rx_buf[self.rx_len..self.rx_len + data_len]
                        .copy_from_slice(&buf[9..9 + data_len]);
                    self.rx_len += data_len;

                    let mut wa = [0u8; 9];
                    encode_window_adjust(&mut wa, self.recipient_channel, data_len as u32)?;
                    self.raw_send(&wa)?;
                }
                APF_CHANNEL_WINDOW_ADJUST => {
                    if len >= 9 {
                        self.tx_window = self.tx_window.saturating_add(read_be32(&buf[5..9]));
                    }
                }
                APF_CHANNEL_CLOSE => {
                    if self.channel_active {
                        let mut close = [0u8; 5];
                        encode_channel_close(&mut close, self.recipient_channel)?;
                        let _ = self.raw_send(&close);
                        self.channel_active = false;
                    }
                    break;
                }
                APF_KEEPALIVE_REQUEST => {
                    if len >= 5 {
                        let cookie = read_be32(&buf[1..5]);
                        let mut reply = [0u8; 5];
                        encode_keepalive_reply(&mut reply, cookie)?;
                        self.raw_send(&reply)?;
                    }
                }
                _ => { /* ignore */ }
            }
        }

        if self.rx_len == 0 {
            return Err(ApfError::Aborted);
        }
        let copy = self.rx_len.min(out.len());
        out[..copy].copy_from_slice(&self.rx_buf[..copy]);
        Ok(copy)
    }

    pub(crate) fn process_apf(&mut self, data: &[u8]) -> Result<u8, ApfError> {
        if data.is_empty() {
            return Err(ApfError::Protocol("empty APF message"));
        }
        let mt = data[0];
        match mt {
            APF_PROTOCOLVERSION => {}
            APF_SERVICE_REQUEST => {
                if data.len() < 5 {
                    return Err(ApfError::Protocol("SERVICE_REQUEST too short"));
                }
                let name_len = read_be32(&data[1..5]) as usize;
                if data.len() < 5 + name_len {
                    return Err(ApfError::Protocol("SERVICE_REQUEST truncated"));
                }
                let name = &data[5..5 + name_len];
                let mut out = [0u8; 128];
                let n = encode_service_accept(&mut out, name)?;
                self.raw_send(&out[..n])?;
            }
            APF_GLOBAL_REQUEST => {
                if data.len() < 5 { return Err(ApfError::Protocol("GLOBAL_REQUEST short")); }
                let name_len = read_be32(&data[1..5]) as usize;
                if data.len() < 6 + name_len {
                    return Err(ApfError::Protocol("GLOBAL_REQUEST truncated"));
                }
                let want_reply = data[5 + name_len];
                let mut offset = 6 + name_len;
                let mut port = 0u32;
                if offset + 4 <= data.len() {
                    let addr_len = read_be32(&data[offset..offset + 4]) as usize;
                    offset += 4 + addr_len;
                    if offset + 4 <= data.len() {
                        port = read_be32(&data[offset..offset + 4]);
                    }
                }
                if want_reply != 0 {
                    let mut out = [0u8; 5];
                    encode_request_success(&mut out, port)?;
                    self.raw_send(&out)?;
                }
                if port == APF_AMT_HTTP_PORT {
                    self.port_forward_ok = true;
                }
            }
            APF_CHANNEL_OPEN_CONFIRMATION | APF_CHANNEL_OPEN_FAILURE
            | APF_CHANNEL_WINDOW_ADJUST | APF_CHANNEL_DATA | APF_CHANNEL_CLOSE
            | APF_KEEPALIVE_REQUEST => {}
            _ => {}
        }
        Ok(mt)
    }

    /// Reopen a channel after the previous one was closed.
    ///
    /// If `channel_open` returns a Heci or Aborted error (some ME firmware
    /// sends HBM_CLIENT_DISCONNECT_REQ instead of CONFIRMATION on a reopen),
    /// invoke `HeciHooks::reconnect_heci` and retry exactly once.
    pub fn reopen_channel(&mut self) -> Result<(), ApfError> {
        match self.channel_open() {
            Ok(()) => Ok(()),
            Err(ApfError::Heci(_)) | Err(ApfError::Aborted) => {
                self.channel_active = false;
                self.recipient_channel = 0;
                self.tx_window = 0;
                self.port_forward_ok = false;
                self.hooks
                    .reconnect_heci(&mut self.heci)
                    .map_err(ApfError::from)?;
                self.force_port_forward_ok();
                self.channel_open()
            }
            Err(e) => Err(e),
        }
    }
}

fn data_first_byte(data: &[u8]) -> Result<u8, ApfError> {
    data.first().copied().ok_or(ApfError::Protocol("empty message"))
}
