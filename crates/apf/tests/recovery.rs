mod common;

use common::{Event, FakeHeci};
use apf::error::HeciError;
use apf::message::{APF_CHANNEL_OPEN, LME_RX_WINDOW_SIZE, write_be32};
use apf::session::ApfSession;
use apf::transport::{HeciHooks, HeciTransport};

const HOST: u8 = 0x07;
const ME: u8 = 0x11;

struct RecordingHook {
    pub calls: std::cell::Cell<u32>,
}
impl HeciHooks for RecordingHook {
    fn reconnect_heci(&mut self, _heci: &mut dyn HeciTransport) -> Result<(), HeciError> {
        self.calls.set(self.calls.get() + 1);
        Ok(())
    }
}

fn bytes_open(sender: u32) -> Vec<u8> {
    let mut v = vec![0u8; 66];
    let mut i = 0;
    v[i] = APF_CHANNEL_OPEN;
    i += 1;
    write_be32(&mut v[i..i + 4], 15);
    i += 4;
    v[i..i + 15].copy_from_slice(b"forwarded-tcpip");
    i += 15;
    write_be32(&mut v[i..i + 4], sender);
    i += 4;
    write_be32(&mut v[i..i + 4], LME_RX_WINDOW_SIZE);
    i += 4;
    write_be32(&mut v[i..i + 4], 0xFFFF_FFFF);
    i += 4;
    write_be32(&mut v[i..i + 4], 9);
    i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1");
    i += 9;
    write_be32(&mut v[i..i + 4], 16992);
    i += 4;
    write_be32(&mut v[i..i + 4], 9);
    i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1");
    i += 9;
    write_be32(&mut v[i..i + 4], 16992);
    v
}

fn bytes_confirmation(recip: u32, sender: u32, window: u32) -> Vec<u8> {
    let mut v = vec![0u8; 17];
    v[0] = apf::message::APF_CHANNEL_OPEN_CONFIRMATION;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], sender);
    write_be32(&mut v[9..13], window);
    v
}

#[test]
fn reopen_retries_after_aborted_open() {
    let script = vec![
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: bytes_open(1),
        },
        Event::ReturnRecvErr(HeciError::Io("HBM disconnect".into())),
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: bytes_open(2),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: bytes_confirmation(2, 77, 4096),
        },
    ];

    let fake = FakeHeci::new(script);
    let hook = RecordingHook {
        calls: std::cell::Cell::new(0),
    };
    let mut s = ApfSession::new(fake, hook, ME, HOST);
    s.force_port_forward_ok();

    s.reopen_channel().expect("reopen ok");
    assert_eq!(s.hooks_ref().calls.get(), 1);
    assert!(s.channel_active());
    assert_eq!(s.recipient_channel(), 77);
}
