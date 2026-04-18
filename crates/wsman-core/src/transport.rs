use crate::error::WsmanError;

/// Sync byte-level transport. Implementors: `ApfTransport`,
/// `ReqwestTransport`, plus test fakes.
pub trait Transport {
    fn post(
        &mut self,
        headers: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError>;
}

/// Caller-owned buffers that the transport fills.
pub struct ResponseBuf<'a> {
    pub body: &'a mut [u8],
    pub body_len: usize,
    pub www_authenticate: &'a mut [u8],
    pub www_authenticate_len: usize,
}

impl<'a> ResponseBuf<'a> {
    pub fn new(body: &'a mut [u8], www_authenticate: &'a mut [u8]) -> Self {
        Self {
            body,
            body_len: 0,
            www_authenticate,
            www_authenticate_len: 0,
        }
    }

    pub fn body_slice(&self) -> &[u8] {
        &self.body[..self.body_len]
    }

    pub fn www_authenticate_slice(&self) -> &[u8] {
        &self.www_authenticate[..self.www_authenticate_len]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResponseMeta {
    pub status: u16,
}
