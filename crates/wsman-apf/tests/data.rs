mod common;

use common::{Event, FakeHeci};
use wsman_apf::message::{APF_CHANNEL_DATA, APF_CHANNEL_WINDOW_ADJUST, write_be32};
use wsman_apf::session::ApfSession;
use wsman_apf::transport::NoHooks;

const HOST: u8 = 0x07;
const ME: u8 = 0x11;

fn bytes_channel_data(recip: u32, data: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 9 + data.len()];
    v[0] = APF_CHANNEL_DATA;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], data.len() as u32);
    v[9..9 + data.len()].copy_from_slice(data);
    v
}

fn bytes_window_adjust(recip: u32, add: u32) -> Vec<u8> {
    let mut v = vec![0u8; 9];
    v[0] = APF_CHANNEL_WINDOW_ADJUST;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], add);
    v
}

fn open_session(script_tail: Vec<Event>) -> ApfSession<FakeHeci, NoHooks> {
    use wsman_apf::message::{
        APF_AMT_HTTP_PORT, APF_CHANNEL_OPEN, APF_CHANNEL_OPEN_CONFIRMATION, LME_RX_WINDOW_SIZE,
    };
    let mut open = vec![0u8; 66];
    {
        let mut i = 0;
        open[i] = APF_CHANNEL_OPEN;
        i += 1;
        write_be32(&mut open[i..i + 4], 15);
        i += 4;
        open[i..i + 15].copy_from_slice(b"forwarded-tcpip");
        i += 15;
        write_be32(&mut open[i..i + 4], 1);
        i += 4;
        write_be32(&mut open[i..i + 4], LME_RX_WINDOW_SIZE);
        i += 4;
        write_be32(&mut open[i..i + 4], 0xFFFF_FFFF);
        i += 4;
        write_be32(&mut open[i..i + 4], 9);
        i += 4;
        open[i..i + 9].copy_from_slice(b"127.0.0.1");
        i += 9;
        write_be32(&mut open[i..i + 4], APF_AMT_HTTP_PORT);
        i += 4;
        write_be32(&mut open[i..i + 4], 9);
        i += 4;
        open[i..i + 9].copy_from_slice(b"127.0.0.1");
        i += 9;
        write_be32(&mut open[i..i + 4], APF_AMT_HTTP_PORT);
    }
    let mut confirm = vec![0u8; 17];
    confirm[0] = APF_CHANNEL_OPEN_CONFIRMATION;
    write_be32(&mut confirm[1..5], 1);
    write_be32(&mut confirm[5..9], 50);
    write_be32(&mut confirm[9..13], 8192);

    let mut script = vec![
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: open,
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: confirm,
        },
    ];
    script.extend(script_tail);

    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);
    session.force_port_forward_ok();
    session.channel_open().unwrap();
    session
}

#[test]
fn send_bytes_wraps_in_channel_data() {
    let payload = b"GET / HTTP/1.1\r\n\r\n";
    let mut s = open_session(vec![Event::ExpectSend {
        me: ME,
        host: HOST,
        data: bytes_channel_data(50, payload),
    }]);
    s.send_bytes(payload).unwrap();
}

#[test]
fn recv_bytes_accumulates_channel_data_and_replies_with_window_adjust() {
    let payload = b"HTTP/1.1 200 OK\r\n\r\nbody";
    let mut s = open_session(vec![
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: bytes_channel_data(1, payload),
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: bytes_window_adjust(50, payload.len() as u32),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: {
                let mut v = vec![0u8; 5];
                v[0] = wsman_apf::message::APF_CHANNEL_CLOSE;
                write_be32(&mut v[1..5], 1);
                v
            },
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: {
                let mut v = vec![0u8; 5];
                v[0] = wsman_apf::message::APF_CHANNEL_CLOSE;
                write_be32(&mut v[1..5], 50);
                v
            },
        },
    ]);

    let mut out = [0u8; 128];
    let n = s.recv_bytes(&mut out).unwrap();
    assert_eq!(&out[..n], payload);
}
