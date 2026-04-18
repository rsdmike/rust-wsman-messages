#![forbid(unsafe_code)]

use reqwest::StatusCode;
use reqwest::blocking::Client as HttpClient;
use wsman_core::error::WsmanError;
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};

pub struct ReqwestTransport {
    http: HttpClient,
    endpoint: String,
}

impl ReqwestTransport {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, WsmanError> {
        let http = HttpClient::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| WsmanError::Transport(e.to_string()))?;
        Ok(Self {
            http,
            endpoint: endpoint.into(),
        })
    }
}

impl Transport for ReqwestTransport {
    fn post(
        &mut self,
        headers: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        let mut req = self.http.post(&self.endpoint).body(body.to_vec());
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        let response = req
            .send()
            .map_err(|e| WsmanError::Transport(e.to_string()))?;

        let status = response.status().as_u16();

        if status == StatusCode::UNAUTHORIZED.as_u16() {
            if let Some(v) = response.headers().get("WWW-Authenticate") {
                let bytes = v.as_bytes();
                if bytes.len() > resp.www_authenticate.len() {
                    return Err(WsmanError::BufferTooSmall {
                        need: bytes.len(),
                        have: resp.www_authenticate.len(),
                    });
                }
                resp.www_authenticate[..bytes.len()].copy_from_slice(bytes);
                resp.www_authenticate_len = bytes.len();
            }
        }

        let body_bytes = response
            .bytes()
            .map_err(|e| WsmanError::Transport(e.to_string()))?;
        if body_bytes.len() > resp.body.len() {
            return Err(WsmanError::BufferTooSmall {
                need: body_bytes.len(),
                have: resp.body.len(),
            });
        }
        resp.body[..body_bytes.len()].copy_from_slice(&body_bytes);
        resp.body_len = body_bytes.len();

        Ok(ResponseMeta { status })
    }
}
