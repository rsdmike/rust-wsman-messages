mod common;

use common::{Event, FakeHeci};
use wsman_apf::message::{
    APF_AMT_HTTP_PORT, APF_GLOBAL_REQUEST, APF_PROTOCOLVERSION, APF_REQUEST_SUCCESS,
    APF_SERVICE_ACCEPT, APF_SERVICE_PFWD, APF_SERVICE_REQUEST, read_be32, write_be32,
};
use wsman_apf::session::ApfSession;
use wsman_apf::transport::NoHooks;

const HOST: u8 = 0x07;
const ME: u8 = 0x11;

fn bytes_proto_version(major: u32, minor: u32) -> Vec<u8> {
    let mut v = vec![0u8; 93];
    v[0] = APF_PROTOCOLVERSION;
    write_be32(&mut v[1..5], major);
    write_be32(&mut v[5..9], minor);
    v
}

fn bytes_service_request(name: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 5 + name.len()];
    v[0] = APF_SERVICE_REQUEST;
    write_be32(&mut v[1..5], name.len() as u32);
    v[5..5 + name.len()].copy_from_slice(name);
    v
}

fn bytes_service_accept(name: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 5 + name.len()];
    v[0] = APF_SERVICE_ACCEPT;
    write_be32(&mut v[1..5], name.len() as u32);
    v[5..5 + name.len()].copy_from_slice(name);
    v
}

fn bytes_global_request_tcpip_forward(port: u32, want_reply: u8) -> Vec<u8> {
    let name = b"tcpip-forward";
    let addr = b"0.0.0.0";
    let mut v = Vec::new();
    v.push(APF_GLOBAL_REQUEST);
    let mut nl = [0u8; 4];
    write_be32(&mut nl, name.len() as u32);
    v.extend_from_slice(&nl);
    v.extend_from_slice(name);
    v.push(want_reply);
    let mut al = [0u8; 4];
    write_be32(&mut al, addr.len() as u32);
    v.extend_from_slice(&al);
    v.extend_from_slice(addr);
    let mut pb = [0u8; 4];
    write_be32(&mut pb, port);
    v.extend_from_slice(&pb);
    v
}

fn bytes_request_success(port: u32) -> Vec<u8> {
    let mut v = vec![0u8; 5];
    v[0] = APF_REQUEST_SUCCESS;
    write_be32(&mut v[1..5], port);
    v
}

#[test]
fn happy_path_handshake() {
    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_proto_version(1, 0) },
        Event::ReturnRecv { me: ME, host: HOST, data: bytes_proto_version(4, 0) },
        Event::ReturnRecv { me: ME, host: HOST, data: bytes_service_request(APF_SERVICE_PFWD) },
        Event::ExpectSend { me: ME, host: HOST, data: bytes_service_accept(APF_SERVICE_PFWD) },
        Event::ReturnRecv { me: ME, host: HOST, data: bytes_global_request_tcpip_forward(APF_AMT_HTTP_PORT, 1) },
        Event::ExpectSend { me: ME, host: HOST, data: bytes_request_success(APF_AMT_HTTP_PORT) },
    ];
    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);

    session.handshake().expect("handshake ok");
    assert!(session.port_forwarding_established());
}

#[test]
fn missing_protocol_version_errors() {
    let script = vec![
        Event::ExpectSend { me: ME, host: HOST, data: bytes_proto_version(1, 0) },
        Event::ReturnRecvErr(wsman_apf::error::HeciError::Io("timeout".into())),
        Event::ReturnRecvErr(wsman_apf::error::HeciError::Io("timeout".into())),
    ];
    let fake = FakeHeci::new(script);
    let mut session = ApfSession::new(fake, NoHooks, ME, HOST);
    assert!(session.handshake().is_err());
}
