use alloc::format;
use alloc::string::String;

use crate::error::WsmanError;
use crate::schema::{Action, DEFAULT_TIMEOUT, WSA_ANONYMOUS};
use crate::selector::Selector;

const XML_PROLOG_OPEN_ENVELOPE: &str = concat!(
    r#"<?xml version="1.0" encoding="utf-8"?>"#,
    r#"<Envelope"#,
    r#" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance""#,
    r#" xmlns:xsd="http://www.w3.org/2001/XMLSchema""#,
    r#" xmlns:a="http://schemas.xmlsoap.org/ws/2004/08/addressing""#,
    r#" xmlns:w="http://schemas.dmtf.org/wbem/wsman/1/wsman.xsd""#,
    r#" xmlns="http://www.w3.org/2003/05/soap-envelope""#,
    r#">"#,
);
const ENVELOPE_END: &str = "</Envelope>";

fn render_header(
    action: Action,
    resource_uri: &str,
    message_id: u64,
    selectors: &[Selector<'_>],
    timeout: Option<&str>,
) -> String {
    let mut s = String::with_capacity(512);
    s.push_str("<Header>");
    s.push_str(&format!("<a:Action>{}</a:Action>", action.uri()));
    s.push_str("<a:To>/wsman</a:To>");
    s.push_str(&format!("<w:ResourceURI>{resource_uri}</w:ResourceURI>"));
    s.push_str(&format!("<a:MessageID>{message_id}</a:MessageID>"));
    s.push_str(&format!(
        "<a:ReplyTo><a:Address>{WSA_ANONYMOUS}</a:Address></a:ReplyTo>"
    ));
    s.push_str(&format!(
        "<w:OperationTimeout>{}</w:OperationTimeout>",
        timeout.unwrap_or(DEFAULT_TIMEOUT)
    ));
    s.push_str(&Selector::render_set(selectors));
    s.push_str("</Header>");
    s
}

fn finish(header: &str, body: &str, out: &mut [u8]) -> Result<usize, WsmanError> {
    let need =
        XML_PROLOG_OPEN_ENVELOPE.len() + header.len() + body.len() + ENVELOPE_END.len();
    if need > out.len() {
        return Err(WsmanError::BufferTooSmall { need, have: out.len() });
    }
    let mut p = 0;
    for chunk in [
        XML_PROLOG_OPEN_ENVELOPE.as_bytes(),
        header.as_bytes(),
        body.as_bytes(),
        ENVELOPE_END.as_bytes(),
    ] {
        out[p..p + chunk.len()].copy_from_slice(chunk);
        p += chunk.len();
    }
    Ok(p)
}

pub fn build_get(
    resource_uri: &str,
    selectors: &[Selector<'_>],
    message_id: u64,
    timeout: Option<&str>,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let h = render_header(Action::Get, resource_uri, message_id, selectors, timeout);
    finish(&h, "<Body></Body>", out)
}

pub fn build_put(
    resource_uri: &str,
    body_xml: &[u8],
    selectors: &[Selector<'_>],
    message_id: u64,
    timeout: Option<&str>,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let h = render_header(Action::Put, resource_uri, message_id, selectors, timeout);
    let body_str = core::str::from_utf8(body_xml)
        .map_err(|_| WsmanError::Builder("body_xml is not UTF-8"))?;
    let body = format!("<Body>{body_str}</Body>");
    finish(&h, &body, out)
}

pub fn build_enumerate(
    resource_uri: &str,
    message_id: u64,
    timeout: Option<&str>,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let h = render_header(Action::Enumerate, resource_uri, message_id, &[], timeout);
    finish(
        &h,
        r#"<Body><Enumerate xmlns="http://schemas.xmlsoap.org/ws/2004/09/enumeration" /></Body>"#,
        out,
    )
}

pub fn build_pull(
    resource_uri: &str,
    enumeration_context: &str,
    message_id: u64,
    timeout: Option<&str>,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let h = render_header(Action::Pull, resource_uri, message_id, &[], timeout);
    let body = format!(
        r#"<Body><Pull xmlns="http://schemas.xmlsoap.org/ws/2004/09/enumeration"><EnumerationContext>{enumeration_context}</EnumerationContext><MaxElements>999</MaxElements><MaxCharacters>99999</MaxCharacters></Pull></Body>"#
    );
    finish(&h, &body, out)
}

pub fn build_invoke(
    action_uri: &'static str,
    resource_uri: &str,
    input_xml: &[u8],
    selectors: &[Selector<'_>],
    message_id: u64,
    timeout: Option<&str>,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let h = render_header(
        Action::Invoke(action_uri),
        resource_uri,
        message_id,
        selectors,
        timeout,
    );
    let input_str = core::str::from_utf8(input_xml)
        .map_err(|_| WsmanError::Builder("input_xml is not UTF-8"))?;
    let body = format!("<Body>{input_str}</Body>");
    finish(&h, &body, out)
}
