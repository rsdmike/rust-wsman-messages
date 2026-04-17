use wsman_core::digest::{build_authorization_header, Challenge};

#[test]
fn authorization_header_matches_fixture() {
    // WWW-Authenticate header captured from real AMT.
    let www_authenticate = r#"Digest realm="Digest:AF541D9BC94CFF7ADFA073F492F42D77", nonce="qnEoIjICAAAAAAAAkR6lyvhHCEXL4y9z", stale="false", qop="auth""#;
    let challenge = Challenge::parse(www_authenticate).expect("parse challenge");

    let header = build_authorization_header(
        &challenge, "admin", "P@ssw0rd", "POST", "/wsman", /* nc */ 1,
        /* cnonce */ "0a4f113b",
    )
    .expect("build header");

    assert!(
        header.starts_with("Digest username=\"admin\""),
        "got: {header}"
    );
    assert!(
        header.contains("realm=\"Digest:AF541D9BC94CFF7ADFA073F492F42D77\""),
        "got: {header}"
    );
    assert!(header.contains("uri=\"/wsman\""), "got: {header}");
    assert!(header.contains("nc=00000001"), "got: {header}");
    assert!(header.contains("cnonce=\"0a4f113b\""), "got: {header}");
    assert!(header.contains("qop=auth"), "got: {header}");
    // Response is MD5 hex — 32 chars.
    let response = extract_attr(&header, "response");
    assert_eq!(response.len(), 32, "md5 hex length; got: {header}");
    assert!(
        response.chars().all(|c| c.is_ascii_hexdigit()),
        "got: {header}"
    );
}

fn extract_attr<'a>(header: &'a str, name: &str) -> &'a str {
    let needle = format!("{name}=\"");
    let start = header.find(&needle).unwrap() + needle.len();
    let rest = &header[start..];
    let end = rest.find('"').unwrap();
    &rest[..end]
}
