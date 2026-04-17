use crate::digest::{build_authorization_header, Challenge};
use crate::error::WsmanError;
use reqwest::StatusCode;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Credentials {
    Digest { username: String, password: String },
}

impl Credentials {
    pub fn digest(username: impl Into<String>, password: impl Into<String>) -> Self {
        Credentials::Digest {
            username: username.into(),
            password: password.into(),
        }
    }
}

pub struct ClientBuilder {
    endpoint: Option<String>,
    credentials: Option<Credentials>,
    accept_invalid_certs: bool,
}

impl ClientBuilder {
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    pub fn credentials(mut self, c: Credentials) -> Self {
        self.credentials = Some(c);
        self
    }

    pub fn accept_invalid_certs(mut self, b: bool) -> Self {
        self.accept_invalid_certs = b;
        self
    }

    pub fn build(self) -> Result<Client, WsmanError> {
        let endpoint = self
            .endpoint
            .ok_or(WsmanError::BuilderMisuse("endpoint required"))?;
        let credentials = self
            .credentials
            .ok_or(WsmanError::BuilderMisuse("credentials required"))?;
        let http = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(self.accept_invalid_certs)
            .build()?;
        let path = url::Url::parse(&endpoint)
            .map_err(|_| WsmanError::BuilderMisuse("invalid endpoint URL"))?
            .path()
            .to_string();
        Ok(Client {
            inner: Arc::new(Inner {
                http,
                endpoint,
                endpoint_path: path,
                credentials,
                message_id: AtomicU64::new(0),
                nc: AtomicU32::new(0),
            }),
        })
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    endpoint: String,
    endpoint_path: String,
    credentials: Credentials,
    message_id: AtomicU64,
    nc: AtomicU32,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            endpoint: None,
            credentials: None,
            accept_invalid_certs: false,
        }
    }

    /// Returns and increments the next message-id (matches Go's `MessageID++`).
    pub fn next_message_id(&self) -> u64 {
        self.inner.message_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Sends one WS-MAN request, handling a single 401 -> digest-retry round trip.
    pub async fn execute(&self, xml_input: &str) -> Result<String, WsmanError> {
        let response = self
            .inner
            .http
            .post(&self.inner.endpoint)
            .header("Content-Type", "application/soap+xml;charset=UTF-8")
            .body(xml_input.to_string())
            .send()
            .await?;

        if response.status() != StatusCode::UNAUTHORIZED {
            return read_body(response).await;
        }

        let www = response
            .headers()
            .get("WWW-Authenticate")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| WsmanError::Auth("401 without WWW-Authenticate".into()))?;
        let challenge = Challenge::parse(www)?;

        let nc = self.inner.nc.fetch_add(1, Ordering::Relaxed) + 1;
        let cnonce = random_cnonce();
        let (user, pw) = match &self.inner.credentials {
            Credentials::Digest { username, password } => (username, password),
        };
        let auth = build_authorization_header(
            &challenge,
            user,
            pw,
            "POST",
            &self.inner.endpoint_path,
            nc,
            &cnonce,
        )?;

        let response = self
            .inner
            .http
            .post(&self.inner.endpoint)
            .header("Content-Type", "application/soap+xml;charset=UTF-8")
            .header("Authorization", auth)
            .body(xml_input.to_string())
            .send()
            .await?;

        read_body(response).await
    }
}

async fn read_body(resp: reqwest::Response) -> Result<String, WsmanError> {
    let status = resp.status();
    let body = resp.text().await?;
    if status.is_success() {
        Ok(body)
    } else {
        Err(WsmanError::InvalidResponse {
            status: status.as_u16(),
            body,
        })
    }
}

fn random_cnonce() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Non-cryptographic; good enough for client nonce. Swap for `rand` if needed later.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:08x}", nanos)
}
