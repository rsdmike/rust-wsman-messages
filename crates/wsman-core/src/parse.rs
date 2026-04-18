use alloc::string::String;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Returns the text content of the first element whose local name matches
/// `local`. Allocates only the returned `String`; ignores attributes.
pub fn extract_text(xml: &[u8], local: &str) -> Option<String> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = alloc::vec::Vec::new();
    let mut inside = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if local_name_matches(e.name().as_ref(), local.as_bytes()) {
                    inside = true;
                }
            }
            Ok(Event::Text(t)) if inside => {
                let raw = t.unescape().ok()?;
                return Some(raw.into_owned());
            }
            Ok(Event::End(e)) if inside => {
                if local_name_matches(e.name().as_ref(), local.as_bytes()) {
                    return None; // empty element
                }
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }
}

pub fn extract_u32(xml: &[u8], local: &str) -> Option<u32> {
    let s = extract_text(xml, local)?;
    s.trim().parse().ok()
}

/// True if the envelope body contains a `Fault` element (SOAP 1.2).
pub fn has_fault(xml: &[u8]) -> bool {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = alloc::vec::Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if local_name_matches(e.name().as_ref(), b"Fault") {
                    return true;
                }
            }
            Ok(Event::Eof) => return false,
            Err(_) => return false,
            _ => {}
        }
        buf.clear();
    }
}

fn local_name_matches(qname: &[u8], local: &[u8]) -> bool {
    // Strip optional `prefix:` from `qname` before comparing.
    let stripped = match qname.iter().position(|&b| b == b':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    };
    stripped == local
}
