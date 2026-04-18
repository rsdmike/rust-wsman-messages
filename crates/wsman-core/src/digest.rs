use alloc::string::{String, ToString};
use md5::{Digest, Md5};

use crate::error::WsmanError;

#[derive(Debug, Clone)]
pub struct Challenge {
    realm: String,
    nonce: String,
    qop: String,
    opaque: String,
}

impl Challenge {
    pub fn realm(&self) -> &str {
        &self.realm
    }
    pub fn nonce(&self) -> &str {
        &self.nonce
    }
    pub fn qop(&self) -> &str {
        &self.qop
    }
    pub fn opaque(&self) -> &str {
        &self.opaque
    }

    pub fn parse(header_value: &[u8]) -> Result<Self, WsmanError> {
        let s = core::str::from_utf8(header_value)
            .map_err(|_| WsmanError::Auth("challenge not UTF-8"))?;
        let s = s.trim();
        let rest = s
            .strip_prefix("Digest ")
            .or_else(|| s.strip_prefix("digest "))
            .ok_or(WsmanError::Auth("not a Digest challenge"))?;

        Ok(Self {
            realm: extract_field(rest, "realm").unwrap_or_default(),
            nonce: extract_field(rest, "nonce").unwrap_or_default(),
            qop: extract_field(rest, "qop").unwrap_or_default(),
            opaque: extract_field(rest, "opaque").unwrap_or_default(),
        })
    }
}

fn extract_field(s: &str, name: &str) -> Option<String> {
    let needle_q = {
        let mut n = String::from(name);
        n.push_str("=\"");
        n
    };
    if let Some(start) = s.find(&needle_q) {
        let begin = start + needle_q.len();
        let end = s[begin..].find('"')?;
        return Some(s[begin..begin + end].to_string());
    }
    let needle = {
        let mut n = String::from(name);
        n.push('=');
        n
    };
    let start = s.find(&needle)?;
    let begin = start + needle.len();
    let tail = &s[begin..];
    let end = tail
        .find([',', ' ', '\r', '\n'])
        .unwrap_or(tail.len());
    Some(tail[..end].to_string())
}

/// Computes the `Authorization: Digest ...` header value (without the
/// leading `Authorization: `) and writes it into `out`. Returns the
/// number of bytes written.
#[allow(clippy::too_many_arguments)]
pub fn build_authorization_header(
    challenge: &Challenge,
    username: &str,
    password: &str,
    method: &str,
    uri: &str,
    nc: u32,
    cnonce: &str,
    out: &mut [u8],
) -> Result<usize, WsmanError> {
    let ha1 = md5_hex(&[
        username.as_bytes(),
        b":",
        challenge.realm.as_bytes(),
        b":",
        password.as_bytes(),
    ]);
    let ha2 = md5_hex(&[method.as_bytes(), b":", uri.as_bytes()]);
    let nc_str = format_nc(nc);

    let response = if challenge.qop.is_empty() {
        md5_hex(&[
            ha1.as_bytes(),
            b":",
            challenge.nonce.as_bytes(),
            b":",
            ha2.as_bytes(),
        ])
    } else {
        md5_hex(&[
            ha1.as_bytes(),
            b":",
            challenge.nonce.as_bytes(),
            b":",
            nc_str.as_bytes(),
            b":",
            cnonce.as_bytes(),
            b":",
            challenge.qop.as_bytes(),
            b":",
            ha2.as_bytes(),
        ])
    };

    let mut rendered = String::with_capacity(512);
    rendered.push_str("Digest username=\"");
    rendered.push_str(username);
    rendered.push_str("\",realm=\"");
    rendered.push_str(&challenge.realm);
    rendered.push_str("\",nonce=\"");
    rendered.push_str(&challenge.nonce);
    rendered.push_str("\",uri=\"");
    rendered.push_str(uri);
    rendered.push_str("\",response=\"");
    rendered.push_str(&response);
    rendered.push('"');

    if !challenge.opaque.is_empty() {
        rendered.push_str(",opaque=\"");
        rendered.push_str(&challenge.opaque);
        rendered.push('"');
    }
    if !challenge.qop.is_empty() {
        rendered.push_str(",qop=\"");
        rendered.push_str(&challenge.qop);
        rendered.push_str("\",nc=\"");
        rendered.push_str(&nc_str);
        rendered.push_str("\",cnonce=\"");
        rendered.push_str(cnonce);
        rendered.push('"');
    }

    let bytes = rendered.as_bytes();
    if bytes.len() > out.len() {
        return Err(WsmanError::BufferTooSmall {
            need: bytes.len(),
            have: out.len(),
        });
    }
    out[..bytes.len()].copy_from_slice(bytes);
    Ok(bytes.len())
}

fn md5_hex(parts: &[&[u8]]) -> String {
    let mut h = Md5::new();
    for p in parts {
        h.update(p);
    }
    let digest = h.finalize();
    let mut s = String::with_capacity(32);
    for b in digest.iter() {
        s.push_str(HEX[(b >> 4) as usize]);
        s.push_str(HEX[(b & 0x0f) as usize]);
    }
    s
}

const HEX: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];

fn format_nc(nc: u32) -> String {
    let mut s = String::with_capacity(8);
    for shift in (0..32).step_by(4).rev() {
        let nib = (nc >> shift) & 0xf;
        s.push_str(HEX[nib as usize]);
    }
    s
}
