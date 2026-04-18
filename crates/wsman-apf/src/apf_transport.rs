use alloc::format;

use wsman_core::error::WsmanError;
use wsman_core::transport::{ResponseBuf, ResponseMeta, Transport};

use crate::error::ApfError;
use crate::session::ApfSession;
use crate::transport::{HeciHooks, HeciTransport};

const HOST_HEADER_HOST: &str = "127.0.0.1:16992";
const REQUEST_URI: &str = "/wsman";
const FRAMING_BUF_SIZE: usize = 3072;
const STREAM_BUF_SIZE: usize = 8192;

pub struct ApfTransport<T: HeciTransport, H: HeciHooks> {
    session: ApfSession<T, H>,
}

impl<T: HeciTransport, H: HeciHooks> ApfTransport<T, H> {
    pub fn new(session: ApfSession<T, H>) -> Self {
        Self { session }
    }

    pub fn session_mut(&mut self) -> &mut ApfSession<T, H> {
        &mut self.session
    }
}

impl<T: HeciTransport, H: HeciHooks> Transport for ApfTransport<T, H> {
    fn post(
        &mut self,
        headers: &[(&str, &str)],
        body: &[u8],
        resp: &mut ResponseBuf<'_>,
    ) -> Result<ResponseMeta, WsmanError> {
        let mut frame = [0u8; FRAMING_BUF_SIZE];
        let req_len = build_http_request(&mut frame, headers, body).map_err(apf_to_wsman)?;

        if !self.session.channel_active() {
            self.session.reopen_channel().map_err(apf_to_wsman)?;
        }
        self.session.send_bytes(&frame[..req_len]).map_err(apf_to_wsman)?;

        let mut stream = [0u8; STREAM_BUF_SIZE];
        let n = self.session.recv_bytes(&mut stream).map_err(apf_to_wsman)?;

        parse_http_response(&stream[..n], resp)
    }
}

fn build_http_request(
    out: &mut [u8],
    extra_headers: &[(&str, &str)],
    body: &[u8],
) -> Result<usize, ApfError> {
    let mut s = alloc::string::String::with_capacity(512);
    s.push_str("POST ");
    s.push_str(REQUEST_URI);
    s.push_str(" HTTP/1.1\r\n");
    s.push_str("Host: ");
    s.push_str(HOST_HEADER_HOST);
    s.push_str("\r\n");
    for (k, v) in extra_headers {
        s.push_str(k);
        s.push_str(": ");
        s.push_str(v);
        s.push_str("\r\n");
    }
    s.push_str(&format!("Content-Length: {}\r\n", body.len()));
    s.push_str("\r\n");

    let header_bytes = s.as_bytes();
    let total = header_bytes.len() + body.len();
    if total > out.len() {
        return Err(ApfError::BufferTooSmall);
    }
    out[..header_bytes.len()].copy_from_slice(header_bytes);
    out[header_bytes.len()..header_bytes.len() + body.len()].copy_from_slice(body);
    Ok(total)
}

fn parse_http_response(
    raw: &[u8],
    out: &mut ResponseBuf<'_>,
) -> Result<ResponseMeta, WsmanError> {
    let prefix = if raw.starts_with(b"HTTP/1.1 ") {
        Some(9)
    } else if raw.starts_with(b"HTTP/1.0 ") {
        Some(9)
    } else {
        None
    }
    .ok_or(WsmanError::Parse("no HTTP status line"))?;

    let mut status = 0u16;
    for i in 0..3 {
        let b = raw[prefix + i];
        if !(b'0'..=b'9').contains(&b) {
            return Err(WsmanError::Parse("bad status digits"));
        }
        status = status * 10 + (b - b'0') as u16;
    }

    let header_end = find_subslice(raw, b"\r\n\r\n")
        .ok_or(WsmanError::Parse("no header terminator"))?;
    let headers = &raw[..header_end];
    let body_start = header_end + 4;

    if let Some(line_start) = find_header(headers, b"www-authenticate:") {
        let value_start = line_start;
        let line_end = headers[value_start..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| value_start + p)
            .unwrap_or(headers.len());
        let value = trim(&headers[value_start..line_end]);
        if value.len() > out.www_authenticate.len() {
            return Err(WsmanError::BufferTooSmall {
                need: value.len(),
                have: out.www_authenticate.len(),
            });
        }
        out.www_authenticate[..value.len()].copy_from_slice(value);
        out.www_authenticate_len = value.len();
    }

    let mut content_length: Option<usize> = None;
    if let Some(line_start) = find_header(headers, b"content-length:") {
        let line_end = headers[line_start..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| line_start + p)
            .unwrap_or(headers.len());
        let value = trim(&headers[line_start..line_end]);
        let s = core::str::from_utf8(value).map_err(|_| WsmanError::Parse("bad content-length"))?;
        content_length =
            Some(s.trim().parse().map_err(|_| WsmanError::Parse("bad content-length"))?);
    }

    let body_full = &raw[body_start..];
    let body = match content_length {
        Some(n) if n <= body_full.len() => &body_full[..n],
        _ => body_full,
    };

    if body.len() > out.body.len() {
        return Err(WsmanError::BufferTooSmall { need: body.len(), have: out.body.len() });
    }
    out.body[..body.len()].copy_from_slice(body);
    out.body_len = body.len();
    Ok(ResponseMeta { status })
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn find_header(raw: &[u8], name_colon_lower: &[u8]) -> Option<usize> {
    let lower: alloc::vec::Vec<u8> = raw.iter().map(|b| b.to_ascii_lowercase()).collect();
    let start = find_subslice(&lower, name_colon_lower)?;
    Some(start + name_colon_lower.len())
}

fn trim(s: &[u8]) -> &[u8] {
    let mut lo = 0;
    let mut hi = s.len();
    while lo < hi && (s[lo] == b' ' || s[lo] == b'\t') {
        lo += 1;
    }
    while hi > lo && (s[hi - 1] == b' ' || s[hi - 1] == b'\t') {
        hi -= 1;
    }
    &s[lo..hi]
}

fn apf_to_wsman(e: ApfError) -> WsmanError {
    use alloc::string::ToString;
    WsmanError::Transport(e.to_string())
}
