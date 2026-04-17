use thiserror::Error;

#[derive(Debug, Clone)]
pub struct SoapFault {
    pub code: String,
    pub subcode: Option<String>,
    pub reason: String,
}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.subcode {
            Some(sc) => write!(f, "SOAP fault [{}/{}]: {}", self.code, sc, self.reason),
            None => write!(f, "SOAP fault [{}]: {}", self.code, self.reason),
        }
    }
}

#[derive(Debug, Error)]
pub enum WsmanError {
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("digest auth failed: {0}")]
    Auth(String),

    #[error("invalid response (HTTP {status}): {body}")]
    InvalidResponse { status: u16, body: String },

    #[error("{0}")]
    SoapFault(SoapFault),

    #[error("XML deserialize: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("builder misuse: {0}")]
    BuilderMisuse(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soap_fault_display() {
        let err = WsmanError::SoapFault(SoapFault {
            code: "s:Sender".into(),
            subcode: Some("wsa:ActionNotSupported".into()),
            reason: "The action is not supported by the service.".into(),
        });
        let msg = format!("{err}");
        assert!(msg.contains("ActionNotSupported"));
        assert!(msg.contains("not supported"));
    }
}
