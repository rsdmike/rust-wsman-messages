use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wsman_core::client::{Client, Credentials};

fn headers_missing_authorization() -> impl wiremock::Match {
    struct M;
    impl wiremock::Match for M {
        fn matches(&self, req: &wiremock::Request) -> bool {
            !req.headers.contains_key("authorization")
        }
    }
    M
}

#[tokio::test]
async fn executes_digest_dance() {
    let server = MockServer::start().await;

    // First request (no auth) -> 401 challenge
    Mock::given(method("POST"))
        .and(path("/wsman"))
        .and(headers_missing_authorization())
        .respond_with(
            ResponseTemplate::new(401).insert_header(
                "WWW-Authenticate",
                r#"Digest realm="Digest:AF541D9BC94CFF7ADFA073F492F42D77", nonce="qnEoIjICAAAAAAAAkR6lyvhHCEXL4y9z", stale="false", qop="auth""#,
            ),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Second request (with Authorization) -> 200
    Mock::given(method("POST"))
        .and(path("/wsman"))
        .and(wiremock::matchers::header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<ok/>"))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .endpoint(format!("{}/wsman", server.uri()))
        .credentials(Credentials::digest("admin", "P@ssw0rd"))
        .build()
        .unwrap();

    let body = client.execute("<request/>").await.unwrap();
    assert_eq!(body, "<ok/>");
}
