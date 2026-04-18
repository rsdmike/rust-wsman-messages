use alloc::string::String;

#[derive(Debug, thiserror::Error)]
pub enum WsmanError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("auth: {0}")]
    Auth(&'static str),
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("malformed XML: {0}")]
    Parse(&'static str),
    #[error("SOAP fault: {0}")]
    SoapFault(String),
    #[error("buffer too small: need {need}, have {have}")]
    BufferTooSmall { need: usize, have: usize },
    #[error("builder: {0}")]
    Builder(&'static str),
}
