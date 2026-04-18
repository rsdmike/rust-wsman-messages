use wsman_core::WsmanError;
use wsman_core::client::{Client, Credentials};
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};

/// Scripted response. `status == 401` also sets `www_authenticate`.
struct Scripted {
    queue: Vec<(u16, Vec<u8>, Vec<u8>)>, // (status, body, www-authenticate)
    last_headers: Vec<Vec<(String, String)>>,
    last_body: Vec<Vec<u8>>,
}

impl Scripted {
    fn new(q: Vec<(u16, &[u8], &[u8])>) -> Self {
        Self {
            queue: q
                .into_iter()
                .map(|(s, b, w)| (s, b.to_vec(), w.to_vec()))
                .collect(),
            last_headers: Vec::new(),
            last_body: Vec::new(),
        }
    }
}

impl Transport for Scripted {
    fn post(
        &mut self,
        headers: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        self.last_headers.push(
            headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        self.last_body.push(body.to_vec());

        let (status, body_bytes, www) = self.queue.remove(0);
        resp.body[..body_bytes.len()].copy_from_slice(&body_bytes);
        resp.body_len = body_bytes.len();
        resp.www_authenticate[..www.len()].copy_from_slice(&www);
        resp.www_authenticate_len = www.len();
        Ok(ResponseMeta { status })
    }
}

#[test]
fn execute_returns_body_on_200() {
    let t = Scripted::new(vec![(200, b"<ok/>", b"")]);
    let mut c = Client::new(t, Credentials::digest("u", "p"));
    let mut body = [0u8; 256];
    let mut auth = [0u8; 256];
    let mut rb = ResponseBuf::new(&mut body, &mut auth);
    let n = c.execute(b"<req/>", &mut rb).unwrap();
    assert_eq!(&rb.body[..n], b"<ok/>");
}

#[test]
fn execute_retries_on_401_then_returns_body() {
    let t = Scripted::new(vec![
        (401, b"", br#"Digest realm="R", nonce="N", qop="auth""#),
        (200, b"<ok/>", b""),
    ]);
    let mut c = Client::new(t, Credentials::digest("user", "pass"));
    let mut body = [0u8; 256];
    let mut auth = [0u8; 256];
    let mut rb = ResponseBuf::new(&mut body, &mut auth);
    let n = c.execute(b"<req/>", &mut rb).unwrap();
    assert_eq!(&rb.body[..n], b"<ok/>");
}

#[test]
fn second_401_is_fatal() {
    let t = Scripted::new(vec![
        (401, b"", br#"Digest realm="R", nonce="N", qop="auth""#),
        (
            401,
            b"still-no",
            br#"Digest realm="R", nonce="N2", qop="auth""#,
        ),
    ]);
    let mut c = Client::new(t, Credentials::digest("u", "p"));
    let mut body = [0u8; 256];
    let mut auth = [0u8; 256];
    let mut rb = ResponseBuf::new(&mut body, &mut auth);
    let err = c.execute(b"<req/>", &mut rb).unwrap_err();
    assert!(matches!(err, WsmanError::Http { status: 401, .. }));
}

#[test]
fn non_200_non_401_is_reported_with_status() {
    let t = Scripted::new(vec![(500, b"internal error", b"")]);
    let mut c = Client::new(t, Credentials::digest("u", "p"));
    let mut body = [0u8; 256];
    let mut auth = [0u8; 256];
    let mut rb = ResponseBuf::new(&mut body, &mut auth);
    let err = c.execute(b"<req/>", &mut rb).unwrap_err();
    match err {
        WsmanError::Http { status, body } => {
            assert_eq!(status, 500);
            assert_eq!(body, "internal error");
        }
        _ => panic!("wrong error: {err:?}"),
    }
}

#[test]
fn message_id_monotonic() {
    let t = Scripted::new(vec![(200, b"a", b""), (200, b"b", b"")]);
    let mut c = Client::new(t, Credentials::digest("u", "p"));
    assert_eq!(c.next_message_id(), 0);
    assert_eq!(c.next_message_id(), 1);
    assert_eq!(c.next_message_id(), 2);
}
