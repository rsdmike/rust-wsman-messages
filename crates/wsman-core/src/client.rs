use alloc::string::{String, ToString};

use crate::digest::{build_authorization_header, Challenge};
use crate::error::WsmanError;
use crate::transport::{ResponseBuf, Transport};

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

pub struct Client<T: Transport> {
    transport: T,
    credentials: Credentials,
    message_id: u64,
    nc: u32,
}

const BASE_HEADERS: [(&str, &str); 2] = [
    ("Content-Type", "application/soap+xml; charset=utf-8"),
    ("Connection", "close"),
];

const REQUEST_URI: &str = "/wsman";

impl<T: Transport> Client<T> {
    pub fn new(transport: T, credentials: Credentials) -> Self {
        Self { transport, credentials, message_id: 0, nc: 0 }
    }

    /// Returns and increments the next message id.
    pub fn next_message_id(&mut self) -> u64 {
        let id = self.message_id;
        self.message_id = self.message_id.wrapping_add(1);
        id
    }

    /// Send `xml`, handle one 401→digest retry, return body length.
    pub fn execute(
        &mut self,
        xml: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<usize, WsmanError> {
        let meta = self.transport.post(&BASE_HEADERS, xml, resp)?;
        match meta.status {
            200 => Ok(resp.body_len),
            401 => self.retry_with_digest(xml, resp),
            status => Err(http_err(status, resp)),
        }
    }

    fn retry_with_digest(
        &mut self,
        xml: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<usize, WsmanError> {
        let www = resp.www_authenticate_slice().to_vec();
        if www.is_empty() {
            return Err(WsmanError::Auth("401 without WWW-Authenticate"));
        }
        let challenge = Challenge::parse(&www)?;
        self.nc = self.nc.wrapping_add(1);
        let cnonce = cnonce_from(self.nc);
        let Credentials::Digest { username, password } = &self.credentials;

        let mut auth_buf = [0u8; 1024];
        let n = build_authorization_header(
            &challenge, username, password, "POST", REQUEST_URI, self.nc, &cnonce, &mut auth_buf,
        )?;
        let auth_str = core::str::from_utf8(&auth_buf[..n])
            .map_err(|_| WsmanError::Auth("auth header not UTF-8"))?;

        let headers: [(&str, &str); 3] = [
            BASE_HEADERS[0],
            BASE_HEADERS[1],
            ("Authorization", auth_str),
        ];

        resp.body_len = 0;
        resp.www_authenticate_len = 0;

        let meta = self.transport.post(&headers, xml, resp)?;
        match meta.status {
            200 => Ok(resp.body_len),
            status => Err(http_err(status, resp)),
        }
    }
}

fn http_err(status: u16, resp: &ResponseBuf<'_>) -> WsmanError {
    let body = core::str::from_utf8(resp.body_slice())
        .unwrap_or("")
        .to_string();
    WsmanError::Http { status, body }
}

/// Deterministic cnonce derived from nc counter. Not cryptographic; AMT
/// doesn't require one to be, and `no_std` has no clock. If stronger
/// values are ever needed, swap this for a caller-provided RNG.
fn cnonce_from(nc: u32) -> String {
    let mut s = String::with_capacity(16);
    let bytes = nc.to_be_bytes();
    for b in bytes {
        s.push_str(HEX[(b >> 4) as usize]);
        s.push_str(HEX[(b & 0x0f) as usize]);
    }
    s.push_str("uefiboot");
    s
}

const HEX: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];
