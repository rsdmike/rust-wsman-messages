use crate::error::WsmanError;
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumerateResponse {
    pub context: String,
}

impl EnumerateResponse {
    pub fn from_envelope(xml: &str) -> Result<Self, WsmanError> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut in_context = false;
        let mut context = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e))
                    if local_name(e.name().as_ref()) == b"EnumerationContext" =>
                {
                    in_context = true;
                }
                Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"EnumerationContext" => {
                    in_context = false;
                }
                Ok(Event::Text(t)) if in_context => {
                    context = Some(t.unescape().map_err(xml_err)?.into_owned());
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(xml_err(e)),
                _ => {}
            }
            buf.clear();
        }

        context
            .map(|c| EnumerateResponse { context: c })
            .ok_or_else(|| WsmanError::BuilderMisuse("no EnumerationContext in response"))
    }
}

fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().position(|&b| b == b':') {
        Some(i) => &name[i + 1..],
        None => name,
    }
}

fn xml_err<E: std::fmt::Display>(e: E) -> WsmanError {
    WsmanError::BuilderMisuse(Box::leak(format!("xml: {e}").into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enumerate_response() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope"
          xmlns:g="http://schemas.xmlsoap.org/ws/2004/09/enumeration">
  <Header/>
  <Body>
    <g:EnumerateResponse>
      <g:EnumerationContext>14000000-0000-0000-0000-000000000000</g:EnumerationContext>
    </g:EnumerateResponse>
  </Body>
</Envelope>"#;

        let resp = EnumerateResponse::from_envelope(xml).unwrap();
        assert_eq!(resp.context, "14000000-0000-0000-0000-000000000000");
    }
}
