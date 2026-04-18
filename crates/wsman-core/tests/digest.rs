use wsman_core::digest::{Challenge, build_authorization_header};

const AMT_401_HEADER: &str =
    r#"Digest realm="Digest:AABBCCDD", nonce="1234567890abcdef", stale="false", qop="auth""#;

#[test]
fn parse_challenge_fields() {
    let c = Challenge::parse(AMT_401_HEADER.as_bytes()).unwrap();
    assert_eq!(c.realm(), "Digest:AABBCCDD");
    assert_eq!(c.nonce(), "1234567890abcdef");
    assert_eq!(c.qop(), "auth");
    assert_eq!(c.opaque(), "");
}

#[test]
fn parse_challenge_with_opaque() {
    let hdr = r#"Digest realm="R", nonce="N", qop="auth", opaque="OPAQUEVAL""#;
    let c = Challenge::parse(hdr.as_bytes()).unwrap();
    assert_eq!(c.opaque(), "OPAQUEVAL");
}

#[test]
fn parse_challenge_missing_prefix_fails() {
    assert!(Challenge::parse(b"Basic realm=\"x\"").is_err());
}

#[test]
fn authorization_header_round_trip() {
    let hdr = r#"Digest realm="realm", nonce="nonceval", qop="auth""#;
    let c = Challenge::parse(hdr.as_bytes()).unwrap();
    let mut out = [0u8; 1024];
    let n = build_authorization_header(
        &c,
        "user",
        "pass",
        "POST",
        "/wsman",
        1,
        "cnonceval",
        &mut out,
    )
    .unwrap();
    let hdr = core::str::from_utf8(&out[..n]).unwrap();

    assert!(hdr.starts_with("Digest "));
    assert!(hdr.contains(r#"username="user""#));
    assert!(hdr.contains(r#"realm="realm""#));
    assert!(hdr.contains(r#"nonce="nonceval""#));
    assert!(hdr.contains(r#"uri="/wsman""#));
    assert!(hdr.contains(r#"qop="auth""#));
    assert!(hdr.contains(r#"nc="00000001""#));
    assert!(hdr.contains(r#"cnonce="cnonceval""#));

    let resp_idx = hdr.find(r#"response=""#).unwrap() + r#"response=""#.len();
    let resp = &hdr[resp_idx..resp_idx + 32];
    assert!(resp.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn authorization_header_without_qop_uses_legacy_form() {
    let hdr = r#"Digest realm="R", nonce="N""#;
    let c = Challenge::parse(hdr.as_bytes()).unwrap();
    let mut out = [0u8; 512];
    let n = build_authorization_header(&c, "u", "p", "POST", "/wsman", 1, "cnonceval", &mut out)
        .unwrap();
    let hdr = core::str::from_utf8(&out[..n]).unwrap();
    assert!(!hdr.contains("qop="));
    assert!(!hdr.contains("nc="));
}

#[test]
fn opaque_included_when_present() {
    let hdr = r#"Digest realm="R", nonce="N", qop="auth", opaque="OP""#;
    let c = Challenge::parse(hdr.as_bytes()).unwrap();
    let mut out = [0u8; 512];
    let n = build_authorization_header(&c, "u", "p", "POST", "/wsman", 1, "cnonceval", &mut out)
        .unwrap();
    let hdr = core::str::from_utf8(&out[..n]).unwrap();
    assert!(hdr.contains(r#"opaque="OP""#));
}
