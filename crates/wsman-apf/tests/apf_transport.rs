mod common;

use common::{Event, FakeHeci};
use wsman_apf::apf_transport::ApfTransport;
use wsman_apf::message::{APF_CHANNEL_DATA, write_be32};
use wsman_apf::session::ApfSession;
use wsman_apf::transport::NoHooks;
use wsman_core::client::{Client, Credentials};
use wsman_core::transport::ResponseBuf;

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

#[test]
fn apf_transport_drives_client_one_roundtrip() {
    let http_response = b"HTTP/1.1 200 OK\r\nContent-Type: application/soap+xml\r\nContent-Length: 5\r\n\r\n<ok/>";
    let close = {
        let mut v = vec![0u8; 5];
        v[0] = wsman_apf::message::APF_CHANNEL_CLOSE;
        write_be32(&mut v[1..5], 1);
        v
    };
    let close_ack = {
        let mut v = vec![0u8; 5];
        v[0] = wsman_apf::message::APF_CHANNEL_CLOSE;
        write_be32(&mut v[1..5], 50);
        v
    };
    let wa = {
        let mut v = vec![0u8; 9];
        v[0] = wsman_apf::message::APF_CHANNEL_WINDOW_ADJUST;
        write_be32(&mut v[1..5], 50);
        write_be32(&mut v[5..9], http_response.len() as u32);
        v
    };

    let request_body = b"<req/>";
    let expected_http = {
        let headers = b"POST /wsman HTTP/1.1\r\nHost: 127.0.0.1:16992\r\nContent-Type: application/soap+xml; charset=utf-8\r\nConnection: close\r\nContent-Length: 6\r\n\r\n";
        let mut v = Vec::from(&headers[..]);
        v.extend_from_slice(request_body);
        v
    };

    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_channel_data(50, &expected_http) },
        Event::ReturnRecv { me: ME, host: HOST, data: bytes_channel_data(1, http_response) },
        Event::ExpectSend { me: ME, host: HOST, data: wa },
        Event::ReturnRecv { me: ME, host: HOST, data: close },
        Event::ExpectSend { me: ME, host: HOST, data: close_ack },
    ];

    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);
    session.force_port_forward_ok();
    session.force_channel_state(50, 8192);

    let transport = ApfTransport::new(session);
    let mut client = Client::new(transport, Credentials::digest("u", "p"));

    let mut body = [0u8; 512];
    let mut www = [0u8; 256];
    let mut rb = ResponseBuf::new(&mut body, &mut www);
    let n = client.execute(request_body, &mut rb).unwrap();
    assert_eq!(&rb.body[..n], b"<ok/>");
}
