//! Hand-rolled XML tag extraction.
//!
//! WSMAN AMT responses are small and structurally predictable. Rather than
//! pulling a full XML parser (and its `std` transitive deps) into the
//! no_std crate graph, we walk the bytes directly. Matches by local name
//! (ignores namespace prefixes) and handles a minimal set of XML entity
//! escapes in text content (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).

use alloc::string::String;

/// Returns the text content of the first element whose local name matches
/// `local`. Returns `None` if not found or the element is self-closing or
/// empty. Ignores attributes. Strips XML entity escapes in the returned text.
pub fn extract_text(xml: &[u8], local: &str) -> Option<String> {
    let local_bytes = local.as_bytes();
    let mut i = 0;
    let len = xml.len();

    while i < len {
        if xml[i] != b'<' {
            i += 1;
            continue;
        }

        // Skip comments, CDATA, and closing/decl tags.
        if i + 1 < len && (xml[i + 1] == b'/' || xml[i + 1] == b'?' || xml[i + 1] == b'!') {
            i += 1;
            continue;
        }

        // Find end of tag name (`>`, space, or `/`).
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        while tag_end < len
            && xml[tag_end] != b'>'
            && xml[tag_end] != b' '
            && xml[tag_end] != b'/'
            && xml[tag_end] != b'\t'
            && xml[tag_end] != b'\r'
            && xml[tag_end] != b'\n'
        {
            tag_end += 1;
        }
        if tag_end >= len {
            return None;
        }

        // Does the tag's local name match?
        if local_name_matches(&xml[tag_start..tag_end], local_bytes) {
            // Skip to the `>` that closes the opening tag.
            let mut gt = tag_end;
            let mut self_closing = false;
            while gt < len && xml[gt] != b'>' {
                if xml[gt] == b'/' {
                    self_closing = true;
                }
                gt += 1;
            }
            if gt >= len {
                return None;
            }
            if self_closing {
                return None; // no text content
            }

            let content_start = gt + 1;
            // Find the next `<` — that's the start of the close tag.
            let mut content_end = content_start;
            while content_end < len && xml[content_end] != b'<' {
                content_end += 1;
            }
            if content_end == content_start {
                return None; // empty element
            }

            return Some(unescape(&xml[content_start..content_end]));
        }

        i = tag_end;
    }
    None
}

/// Parse the text content of the first element with local name `local` as
/// a decimal `u32`. Strips leading/trailing ASCII whitespace.
pub fn extract_u32(xml: &[u8], local: &str) -> Option<u32> {
    let s = extract_text(xml, local)?;
    s.trim().parse().ok()
}

/// True if the document contains any element whose local name is `Fault`
/// (SOAP 1.2 fault indicator).
pub fn has_fault(xml: &[u8]) -> bool {
    let needle = b"Fault";
    let mut i = 0;
    let len = xml.len();

    while i < len {
        if xml[i] != b'<' {
            i += 1;
            continue;
        }
        if i + 1 < len && (xml[i + 1] == b'/' || xml[i + 1] == b'?' || xml[i + 1] == b'!') {
            i += 1;
            continue;
        }
        let tag_start = i + 1;
        let mut tag_end = tag_start;
        while tag_end < len
            && xml[tag_end] != b'>'
            && xml[tag_end] != b' '
            && xml[tag_end] != b'/'
            && xml[tag_end] != b'\t'
            && xml[tag_end] != b'\r'
            && xml[tag_end] != b'\n'
        {
            tag_end += 1;
        }
        if local_name_matches(&xml[tag_start..tag_end], needle) {
            return true;
        }
        if tag_end >= len {
            return false;
        }
        i = tag_end;
    }
    false
}

/// True if `qname`'s local name (after any `:`) equals `local`.
fn local_name_matches(qname: &[u8], local: &[u8]) -> bool {
    let stripped = match qname.iter().position(|&b| b == b':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    };
    stripped == local
}

/// Minimal XML entity unescape: `&amp;` `&lt;` `&gt;` `&quot;` `&apos;`.
/// Unknown entities pass through verbatim. Preserves multi-byte UTF-8.
fn unescape(text: &[u8]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i] == b'&' {
            if let Some((repl, consumed)) = match_entity(&text[i..]) {
                out.push(repl);
                i += consumed;
                continue;
            }
        }
        // Find the next UTF-8 char boundary.
        let char_len = utf8_char_len(text[i]);
        let end = (i + char_len).min(text.len());
        if let Ok(s) = core::str::from_utf8(&text[i..end]) {
            out.push_str(s);
        } else {
            // Malformed UTF-8 — replace with U+FFFD.
            out.push('\u{FFFD}');
        }
        i = end;
    }
    out
}

fn utf8_char_len(first_byte: u8) -> usize {
    // < 0x80: ASCII. 0x80..0xC0: stray continuation byte (invalid as first); advance by 1.
    if first_byte < 0xC0 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}

/// If `bytes` starts with a known entity, return `(replacement_char, length)`.
fn match_entity(bytes: &[u8]) -> Option<(char, usize)> {
    if bytes.starts_with(b"&amp;") {
        return Some(('&', 5));
    }
    if bytes.starts_with(b"&lt;") {
        return Some(('<', 4));
    }
    if bytes.starts_with(b"&gt;") {
        return Some(('>', 4));
    }
    if bytes.starts_with(b"&quot;") {
        return Some(('"', 6));
    }
    if bytes.starts_with(b"&apos;") {
        return Some(('\'', 6));
    }
    None
}
