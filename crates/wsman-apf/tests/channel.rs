mod common;

use common::{Event, FakeHeci};
use wsman_apf::message::{
    APF_AMT_HTTP_PORT, APF_CHANNEL_OPEN, APF_CHANNEL_OPEN_CONFIRMATION,
    APF_CHANNEL_OPEN_FAILURE, LME_RX_WINDOW_SIZE, read_be32, write_be32,
};
use wsman_apf::session::ApfSession;
use wsman_apf::transport::{HeciHooks, NoHooks};

const HOST: u8 = 0x07;
const ME: u8 = 0x11;

fn bytes_channel_open_exp() -> Vec<u8> {
    let mut v = vec![0u8; 66];
    let mut i = 0;
    v[i] = APF_CHANNEL_OPEN; i += 1;
    write_be32(&mut v[i..i + 4], b"forwarded-tcpip".len() as u32); i += 4;
    v[i..i + 15].copy_from_slice(b"forwarded-tcpip"); i += 15;
    write_be32(&mut v[i..i + 4], 1); i += 4;
    write_be32(&mut v[i..i + 4], LME_RX_WINDOW_SIZE); i += 4;
    write_be32(&mut v[i..i + 4], 0xFFFF_FFFF); i += 4;
    write_be32(&mut v[i..i + 4], 9); i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1"); i += 9;
    write_be32(&mut v[i..i + 4], APF_AMT_HTTP_PORT); i += 4;
    write_be32(&mut v[i..i + 4], 9); i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1"); i += 9;
    write_be32(&mut v[i..i + 4], APF_AMT_HTTP_PORT);
    v
}

fn bytes_channel_open_confirmation(recip: u32, sender: u32, window: u32) -> Vec<u8> {
    let mut v = vec![0u8; 17];
    v[0] = APF_CHANNEL_OPEN_CONFIRMATION;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], sender);
    write_be32(&mut v[9..13], window);
    v
}

fn bytes_channel_open_failure(recip: u32, reason: u32) -> Vec<u8> {
    let mut v = vec![0u8; 17];
    v[0] = APF_CHANNEL_OPEN_FAILURE;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], reason);
    v
}

#[test]
fn channel_open_happy_path_records_state() {
    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_channel_open_exp() },
        Event::ReturnRecv {
            me: ME, host: HOST,
            data: bytes_channel_open_confirmation(1, 99, 8192),
        },
    ];
    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);
    session.force_port_forward_ok();

    session.channel_open().expect("open ok");
    assert!(session.channel_active());
    assert_eq!(session.recipient_channel(), 99);
    assert_eq!(session.tx_window(), 8192);
}

#[test]
fn channel_open_failure_returns_error() {
    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_channel_open_exp() },
        Event::ReturnRecv {
            me: ME, host: HOST,
            data: bytes_channel_open_failure(1, 4),
        },
    ];
    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);
    session.force_port_forward_ok();
    let err = session.channel_open().unwrap_err();
    assert!(matches!(err, wsman_apf::error::ApfError::OpenRejected(4)));
}

#[test]
fn post_channel_open_send_hook_fires() {
    struct FlagHook(core::cell::Cell<bool>);
    impl HeciHooks for FlagHook {
        fn post_channel_open_send(&mut self) {
            self.0.set(true);
        }
    }
    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_channel_open_exp() },
        Event::ReturnRecv {
            me: ME, host: HOST,
            data: bytes_channel_open_confirmation(1, 99, 8192),
        },
    ];
    let fake = FakeHeci::new(script);
    let hook = FlagHook(core::cell::Cell::new(false));
    let mut session = ApfSession::new(fake, hook, ME, HOST);
    session.force_port_forward_ok();
    session.channel_open().unwrap();
    assert!(session.hooks_ref().0.get(), "hook should have fired");
}
