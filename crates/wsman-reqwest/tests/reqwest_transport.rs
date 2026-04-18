use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wsman_core::transport::{ResponseBuf, Transport};
use wsman_reqwest::ReqwestTransport;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transport_posts_and_reads_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/wsman"))
        .and(header("Content-Type", "application/soap+xml; charset=utf-8"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(b"<ok/>".to_vec(), "application/soap+xml"),
        )
        .mount(&server)
        .await;

    let endpoint = format!("{}/wsman", server.uri());
    let result = tokio::task::spawn_blocking(move || {
        let mut t = ReqwestTransport::new(&endpoint).unwrap();
        let mut body = [0u8; 256];
        let mut auth = [0u8; 256];
        let mut rb = ResponseBuf::new(&mut body, &mut auth);
        let meta = t
            .post(
                &[("Content-Type", "application/soap+xml; charset=utf-8")],
                b"<req/>",
                &mut rb,
            )
            .unwrap();
        (meta.status, rb.body[..rb.body_len].to_vec())
    })
    .await
    .unwrap();

    assert_eq!(result.0, 200);
    assert_eq!(result.1, b"<ok/>");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn transport_extracts_www_authenticate_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).insert_header(
            "WWW-Authenticate",
            r#"Digest realm="R", nonce="N", qop="auth""#,
        ))
        .mount(&server)
        .await;

    let endpoint = format!("{}/wsman", server.uri());
    let result = tokio::task::spawn_blocking(move || {
        let mut t = ReqwestTransport::new(&endpoint).unwrap();
        let mut body = [0u8; 128];
        let mut auth = [0u8; 256];
        let mut rb = ResponseBuf::new(&mut body, &mut auth);
        let meta = t.post(&[], b"<req/>", &mut rb).unwrap();
        (meta.status, rb.www_authenticate[..rb.www_authenticate_len].to_vec())
    })
    .await
    .unwrap();

    assert_eq!(result.0, 401);
    assert!(std::str::from_utf8(&result.1).unwrap().contains(r#"realm="R""#));
}
