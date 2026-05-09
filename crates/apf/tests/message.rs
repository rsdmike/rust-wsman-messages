use apf::message::*;

#[test]
fn opcode_constants() {
    assert_eq!(APF_SERVICE_REQUEST, 5);
    assert_eq!(APF_SERVICE_ACCEPT, 6);
    assert_eq!(APF_GLOBAL_REQUEST, 80);
    assert_eq!(APF_REQUEST_SUCCESS, 81);
    assert_eq!(APF_CHANNEL_OPEN, 90);
    assert_eq!(APF_CHANNEL_OPEN_CONFIRMATION, 91);
    assert_eq!(APF_CHANNEL_OPEN_FAILURE, 92);
    assert_eq!(APF_CHANNEL_WINDOW_ADJUST, 93);
    assert_eq!(APF_CHANNEL_DATA, 94);
    assert_eq!(APF_CHANNEL_CLOSE, 97);
    assert_eq!(APF_PROTOCOLVERSION, 192);
    assert_eq!(APF_KEEPALIVE_REQUEST, 208);
    assert_eq!(APF_KEEPALIVE_REPLY, 209);
}

#[test]
fn be32_roundtrip() {
    let mut b = [0u8; 4];
    write_be32(&mut b, 0x12345678);
    assert_eq!(b, [0x12, 0x34, 0x56, 0x78]);
    assert_eq!(read_be32(&b), 0x12345678);
}

#[test]
fn encode_protocol_version_msg() {
    let mut out = [0u8; 93];
    let n = encode_protocol_version(&mut out, 1, 0).unwrap();
    assert_eq!(n, 93);
    assert_eq!(out[0], APF_PROTOCOLVERSION);
    assert_eq!(read_be32(&out[1..5]), 1);
    assert_eq!(read_be32(&out[5..9]), 0);
}

#[test]
fn encode_channel_open_forwarded_tcpip() {
    let mut out = [0u8; 72];
    let n = encode_channel_open(&mut out, 1, 4096, 16992).unwrap();
    // Layout: 1 + 4 + 15 + 4 + 4 + 4 + 4 + 9 + 4 + 4 + 9 + 4 = 66 bytes.
    // The caller buffer can be larger; the function returns the exact
    // filled length so APF framing doesn't send trailing zeros.
    assert_eq!(n, 66);
    assert_eq!(out[0], APF_CHANNEL_OPEN);
    assert_eq!(read_be32(&out[1..5]), 15);
    assert_eq!(&out[5..20], b"forwarded-tcpip");
    assert_eq!(read_be32(&out[20..24]), 1);
    assert_eq!(read_be32(&out[24..28]), 4096);
}

#[test]
fn encode_channel_data_frame() {
    let payload = b"GET / HTTP/1.1\r\n\r\n";
    let mut out = [0u8; 256];
    let n = encode_channel_data(&mut out, 5, payload).unwrap();
    assert_eq!(n, 9 + payload.len());
    assert_eq!(out[0], APF_CHANNEL_DATA);
    assert_eq!(read_be32(&out[1..5]), 5);
    assert_eq!(read_be32(&out[5..9]), payload.len() as u32);
    assert_eq!(&out[9..9 + payload.len()], payload);
}

#[test]
fn encode_window_adjust_frame() {
    let mut out = [0u8; 9];
    let n = encode_window_adjust(&mut out, 7, 1024).unwrap();
    assert_eq!(n, 9);
    assert_eq!(out[0], APF_CHANNEL_WINDOW_ADJUST);
    assert_eq!(read_be32(&out[1..5]), 7);
    assert_eq!(read_be32(&out[5..9]), 1024);
}

#[test]
fn encode_service_accept_echoes_name() {
    let name = b"pfwd@amt.intel.com";
    let mut out = [0u8; 64];
    let n = encode_service_accept(&mut out, name).unwrap();
    assert_eq!(out[0], APF_SERVICE_ACCEPT);
    assert_eq!(read_be32(&out[1..5]), name.len() as u32);
    assert_eq!(&out[5..5 + name.len()], name);
    assert_eq!(n, 5 + name.len());
}

#[test]
fn encode_request_success_for_port_forward() {
    let mut out = [0u8; 5];
    let n = encode_request_success(&mut out, 16992).unwrap();
    assert_eq!(n, 5);
    assert_eq!(out[0], APF_REQUEST_SUCCESS);
    assert_eq!(read_be32(&out[1..5]), 16992);
}

#[test]
fn encode_keepalive_reply_copies_cookie() {
    let mut out = [0u8; 5];
    let n = encode_keepalive_reply(&mut out, 0xDEADBEEF).unwrap();
    assert_eq!(n, 5);
    assert_eq!(out[0], APF_KEEPALIVE_REPLY);
    assert_eq!(read_be32(&out[1..5]), 0xDEADBEEF);
}

#[test]
fn encode_channel_close_frame() {
    let mut out = [0u8; 5];
    let n = encode_channel_close(&mut out, 42).unwrap();
    assert_eq!(n, 5);
    assert_eq!(out[0], APF_CHANNEL_CLOSE);
    assert_eq!(read_be32(&out[1..5]), 42);
}
