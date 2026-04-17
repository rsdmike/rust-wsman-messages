use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wsman_amt::general::Settings;
use wsman_core::client::{Client, Credentials};

fn challenge_header() -> (&'static str, &'static str) {
    (
        "WWW-Authenticate",
        r#"Digest realm="Digest:AF541D9BC94CFF7ADFA073F492F42D77", nonce="qnEoIjICAAAAAAAAkR6lyvhHCEXL4y9z", stale="false", qop="auth""#,
    )
}

fn unauthenticated() -> impl wiremock::Match {
    struct M;
    impl wiremock::Match for M {
        fn matches(&self, req: &wiremock::Request) -> bool {
            !req.headers.contains_key("authorization")
        }
    }
    M
}

#[tokio::test]
async fn get_general_settings_end_to_end() {
    let server = MockServer::start().await;

    let get_response = std::fs::read_to_string("tests/fixtures/get.xml").unwrap();

    let (k, v) = challenge_header();
    Mock::given(method("POST"))
        .and(path("/wsman"))
        .and(unauthenticated())
        .respond_with(ResponseTemplate::new(401).insert_header(k, v))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/wsman"))
        .and(wiremock::matchers::header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_string(get_response))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .endpoint(format!("{}/wsman", server.uri()))
        .credentials(Credentials::digest("admin", "P@ssw0rd"))
        .build()
        .unwrap();

    let settings = Settings::new(client);
    let resp = settings.get().await.unwrap();

    assert_eq!(resp.host_name, "Test Host Name");
    assert_eq!(resp.instance_id, "Intel(r) AMT: General Settings");
}
