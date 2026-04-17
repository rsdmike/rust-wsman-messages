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

/// Builds the `<Header>...</Header>` fragment. `reply_to` and `timeout`
/// default to the anonymous address and PT60S when None.
fn header(
    action: Action,
    resource_uri: &str,
    message_id: u64,
    selectors: &[Selector],
    reply_to: Option<&str>,
    timeout: Option<&str>,
) -> String {
    let mut s = String::from("<Header>");
    s.push_str(&format!("<a:Action>{}</a:Action>", action.uri()));
    s.push_str("<a:To>/wsman</a:To>");
    s.push_str(&format!("<w:ResourceURI>{resource_uri}</w:ResourceURI>"));
    s.push_str(&format!("<a:MessageID>{message_id}</a:MessageID>"));
    s.push_str("<a:ReplyTo>");
    s.push_str(&format!(
        "<a:Address>{}</a:Address>",
        reply_to.unwrap_or(WSA_ANONYMOUS)
    ));
    s.push_str("</a:ReplyTo>");
    s.push_str(&format!(
        "<w:OperationTimeout>{}</w:OperationTimeout>",
        timeout.unwrap_or(DEFAULT_TIMEOUT)
    ));
    s.push_str(&Selector::render_set(selectors));
    s.push_str("</Header>");
    s
}

fn wrap(header: &str, body: &str) -> String {
    let mut s = String::with_capacity(
        XML_PROLOG_OPEN_ENVELOPE.len() + header.len() + body.len() + ENVELOPE_END.len(),
    );
    s.push_str(XML_PROLOG_OPEN_ENVELOPE);
    s.push_str(header);
    s.push_str(body);
    s.push_str(ENVELOPE_END);
    s
}

pub fn build_get(
    resource_uri: &str,
    selectors: &[Selector],
    message_id: u64,
    timeout: Option<&str>,
) -> String {
    let h = header(
        Action::Get,
        resource_uri,
        message_id,
        selectors,
        None,
        timeout,
    );
    wrap(&h, "<Body></Body>")
}

pub fn build_enumerate(
    resource_uri: &str,
    selectors: &[Selector],
    message_id: u64,
    timeout: Option<&str>,
) -> String {
    let h = header(
        Action::Enumerate,
        resource_uri,
        message_id,
        selectors,
        None,
        timeout,
    );
    wrap(
        &h,
        r#"<Body><Enumerate xmlns="http://schemas.xmlsoap.org/ws/2004/09/enumeration" /></Body>"#,
    )
}

pub fn build_pull(
    resource_uri: &str,
    enumeration_context: &str,
    selectors: &[Selector],
    message_id: u64,
    timeout: Option<&str>,
) -> String {
    let h = header(
        Action::Pull,
        resource_uri,
        message_id,
        selectors,
        None,
        timeout,
    );
    let body = format!(
        r#"<Body><Pull xmlns="http://schemas.xmlsoap.org/ws/2004/09/enumeration"><EnumerationContext>{enumeration_context}</EnumerationContext><MaxElements>999</MaxElements><MaxCharacters>99999</MaxCharacters></Pull></Body>"#
    );
    wrap(&h, &body)
}

pub fn build_put(
    resource_uri: &str,
    body_xml: &str,
    selectors: &[Selector],
    message_id: u64,
    timeout: Option<&str>,
) -> String {
    let h = header(
        Action::Put,
        resource_uri,
        message_id,
        selectors,
        None,
        timeout,
    );
    let body = format!("<Body>{body_xml}</Body>");
    wrap(&h, &body)
}
