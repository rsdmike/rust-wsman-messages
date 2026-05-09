use alloc::string::String;

#[derive(Debug, Clone, thiserror::Error)]
pub enum HeciError {
    #[error("heci: {0}")]
    Io(String),
    #[error("heci: device busy")]
    Busy,
    #[error("heci: buffer too small")]
    BufferTooSmall,
}

#[derive(Debug, thiserror::Error)]
pub enum ApfError {
    #[error("heci: {0}")]
    Heci(#[from] HeciError),
    #[error("apf protocol: {0}")]
    Protocol(&'static str),
    #[error("apf channel open rejected (reason={0})")]
    OpenRejected(u32),
    #[error("apf timeout waiting for {0}")]
    Timeout(&'static str),
    #[error("apf channel not active")]
    ChannelClosed,
    #[error("apf aborted by ME")]
    Aborted,
    #[error("apf buffer too small")]
    BufferTooSmall,
}
