use crate::error::ApfError;

pub const APF_DISCONNECT: u8 = 1;
pub const APF_SERVICE_REQUEST: u8 = 5;
pub const APF_SERVICE_ACCEPT: u8 = 6;
pub const APF_GLOBAL_REQUEST: u8 = 80;
pub const APF_REQUEST_SUCCESS: u8 = 81;
pub const APF_REQUEST_FAILURE: u8 = 82;
pub const APF_CHANNEL_OPEN: u8 = 90;
pub const APF_CHANNEL_OPEN_CONFIRMATION: u8 = 91;
pub const APF_CHANNEL_OPEN_FAILURE: u8 = 92;
pub const APF_CHANNEL_WINDOW_ADJUST: u8 = 93;
pub const APF_CHANNEL_DATA: u8 = 94;
pub const APF_CHANNEL_CLOSE: u8 = 97;
pub const APF_PROTOCOLVERSION: u8 = 192;
pub const APF_KEEPALIVE_REQUEST: u8 = 208;
pub const APF_KEEPALIVE_REPLY: u8 = 209;

pub const APF_AMT_HTTP_PORT: u32 = 16992;
pub const LME_RX_WINDOW_SIZE: u32 = 4096;

pub const APF_SERVICE_PFWD: &[u8] = b"pfwd@amt.intel.com";
pub const APF_OPEN_CHANNEL_REQUEST_FORWARDED: &[u8] = b"forwarded-tcpip";

/// LME client UUID: {6733A4DB-0476-4E7B-B3AF-BCFC29BEE7A7}
pub const LME_UUID: [u8; 16] = [
    0xdb, 0xa4, 0x33, 0x67, 0x76, 0x04, 0x7b, 0x4e, 0xb3, 0xaf, 0xbc, 0xfc, 0x29, 0xbe, 0xe7, 0xa7,
];

#[inline]
pub fn read_be32(p: &[u8]) -> u32 {
    u32::from_be_bytes([p[0], p[1], p[2], p[3]])
}

#[inline]
pub fn write_be32(buf: &mut [u8], val: u32) {
    let b = val.to_be_bytes();
    buf[0] = b[0];
    buf[1] = b[1];
    buf[2] = b[2];
    buf[3] = b[3];
}

fn require(buf: &[u8], need: usize) -> Result<(), ApfError> {
    if buf.len() < need {
        Err(ApfError::BufferTooSmall)
    } else {
        Ok(())
    }
}

pub fn encode_protocol_version(buf: &mut [u8], major: u32, minor: u32) -> Result<usize, ApfError> {
    require(buf, 93)?;
    buf.iter_mut().take(93).for_each(|b| *b = 0);
    buf[0] = APF_PROTOCOLVERSION;
    write_be32(&mut buf[1..5], major);
    write_be32(&mut buf[5..9], minor);
    Ok(93)
}

pub fn encode_service_accept(buf: &mut [u8], name: &[u8]) -> Result<usize, ApfError> {
    let total = 5 + name.len();
    require(buf, total)?;
    buf[0] = APF_SERVICE_ACCEPT;
    write_be32(&mut buf[1..5], name.len() as u32);
    buf[5..5 + name.len()].copy_from_slice(name);
    Ok(total)
}

pub fn encode_request_success(buf: &mut [u8], port: u32) -> Result<usize, ApfError> {
    require(buf, 5)?;
    buf[0] = APF_REQUEST_SUCCESS;
    write_be32(&mut buf[1..5], port);
    Ok(5)
}

/// Build a `CHANNEL_OPEN` message for `forwarded-tcpip` targeting
/// `127.0.0.1:port`. Fixed 72-byte layout.
pub fn encode_channel_open(
    buf: &mut [u8],
    sender_channel: u32,
    initial_window: u32,
    port: u32,
) -> Result<usize, ApfError> {
    require(buf, 66)?;
    let mut p = 0;
    buf[p] = APF_CHANNEL_OPEN;
    p += 1;
    write_be32(
        &mut buf[p..p + 4],
        APF_OPEN_CHANNEL_REQUEST_FORWARDED.len() as u32,
    );
    p += 4;
    buf[p..p + 15].copy_from_slice(APF_OPEN_CHANNEL_REQUEST_FORWARDED);
    p += 15;
    write_be32(&mut buf[p..p + 4], sender_channel);
    p += 4;
    write_be32(&mut buf[p..p + 4], initial_window);
    p += 4;
    write_be32(&mut buf[p..p + 4], 0xFFFF_FFFF);
    p += 4;
    write_be32(&mut buf[p..p + 4], 9);
    p += 4;
    buf[p..p + 9].copy_from_slice(b"127.0.0.1");
    p += 9;
    write_be32(&mut buf[p..p + 4], port);
    p += 4;
    write_be32(&mut buf[p..p + 4], 9);
    p += 4;
    buf[p..p + 9].copy_from_slice(b"127.0.0.1");
    p += 9;
    write_be32(&mut buf[p..p + 4], port);
    p += 4;
    Ok(p)
}

pub fn encode_channel_data(buf: &mut [u8], recipient: u32, data: &[u8]) -> Result<usize, ApfError> {
    let total = 9 + data.len();
    require(buf, total)?;
    buf[0] = APF_CHANNEL_DATA;
    write_be32(&mut buf[1..5], recipient);
    write_be32(&mut buf[5..9], data.len() as u32);
    buf[9..9 + data.len()].copy_from_slice(data);
    Ok(total)
}

pub fn encode_window_adjust(
    buf: &mut [u8],
    recipient: u32,
    bytes_to_add: u32,
) -> Result<usize, ApfError> {
    require(buf, 9)?;
    buf[0] = APF_CHANNEL_WINDOW_ADJUST;
    write_be32(&mut buf[1..5], recipient);
    write_be32(&mut buf[5..9], bytes_to_add);
    Ok(9)
}

pub fn encode_channel_close(buf: &mut [u8], recipient: u32) -> Result<usize, ApfError> {
    require(buf, 5)?;
    buf[0] = APF_CHANNEL_CLOSE;
    write_be32(&mut buf[1..5], recipient);
    Ok(5)
}

pub fn encode_keepalive_reply(buf: &mut [u8], cookie: u32) -> Result<usize, ApfError> {
    require(buf, 5)?;
    buf[0] = APF_KEEPALIVE_REPLY;
    write_be32(&mut buf[1..5], cookie);
    Ok(5)
}
