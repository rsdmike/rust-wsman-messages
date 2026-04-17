use crate::error::WsmanError;

/// Parsed `WWW-Authenticate: Digest ...` challenge.
pub struct Challenge {
    inner: digest_auth::WwwAuthenticateHeader,
}

impl Challenge {
    /// Parses a `WWW-Authenticate` header value (without the leading header name).
    pub fn parse(header_value: &str) -> Result<Self, WsmanError> {
        let inner = digest_auth::parse(header_value)
            .map_err(|e| WsmanError::Auth(format!("parse challenge: {e}")))?;
        Ok(Self { inner })
    }
}

/// Builds an `Authorization: Digest ...` header value.
///
/// `nc` is the request counter (starts at 1, increments per request with the
/// same nonce). `cnonce` is a client-chosen random string; tests pass a fixed
/// value, production passes a fresh random one.
pub fn build_authorization_header(
    challenge: &Challenge,
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    nc: u32,
    cnonce: &str,
) -> Result<String, WsmanError> {
    let mut ctx = digest_auth::AuthContext::new_with_method(
        username,
        password,
        uri,
        None::<&[u8]>,
        http_method(method),
    );
    ctx.set_custom_cnonce(cnonce.to_string());

    let mut header = challenge.inner.clone();
    // digest_auth maintains nc internally; advance it to `nc`.
    for _ in 1..nc {
        let _ = header.respond(&ctx).map_err(map_err)?;
    }
    let response = header.respond(&ctx).map_err(map_err)?;
    Ok(response.to_header_string())
}

fn http_method(m: &str) -> digest_auth::HttpMethod<'static> {
    match m.to_ascii_uppercase().as_str() {
        "GET" => digest_auth::HttpMethod::GET,
        "POST" => digest_auth::HttpMethod::POST,
        "PUT" => digest_auth::HttpMethod::PUT,
        "DELETE" => digest_auth::HttpMethod::DELETE,
        "HEAD" => digest_auth::HttpMethod::HEAD,
        "OPTIONS" => digest_auth::HttpMethod::OPTIONS,
        "PATCH" => digest_auth::HttpMethod::PATCH,
        other => digest_auth::HttpMethod::from(other.to_string()),
    }
}

fn map_err(e: digest_auth::Error) -> WsmanError {
    WsmanError::Auth(e.to_string())
}
