//! Minimal HTTP/1.1 request parser (request line, headers, Content-Length body).

use crate::limits::{MAX_BODY_BYTES, MAX_HEADER_BYTES};
use std::io::{self, ErrorKind};

/// Parsed inbound HTTP/1.1 request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequest {
    pub method: String,
    /// Path with query string stripped.
    pub path: String,
    pub body: Vec<u8>,
    pub bearer: Option<String>,
    pub accept: Option<String>,
    pub content_type: Option<String>,
    pub keep_alive: bool,
}

/// Parse one HTTP/1.1 request from a complete header+body buffer.
pub fn parse_http_request(raw: &[u8]) -> Result<ParsedRequest, ParseError> {
    let header_end = find_header_end(raw).ok_or(ParseError::Incomplete)?;
    if header_end > MAX_HEADER_BYTES {
        return Err(ParseError::HeadersTooLarge);
    }
    let header_bytes = &raw[..header_end];
    let header_text = std::str::from_utf8(header_bytes).map_err(|_| ParseError::BadRequest)?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or(ParseError::BadRequest)?;
    let (method, target) = parse_request_line(request_line)?;
    let path = strip_query(target).to_string();

    let mut content_length: Option<usize> = None;
    let mut bearer = None;
    let mut accept = None;
    let mut content_type = None;
    let mut connection = None;
    let mut http_version = "HTTP/1.1";
    let mut parts = request_line.split_whitespace();
    let _ = parts.next();
    let _ = parts.next();
    if let Some(v) = parts.next() {
        http_version = v;
    }

    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = split_header(line).ok_or(ParseError::BadRequest)?;
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "content-length" => {
                let n: usize = value.trim().parse().map_err(|_| ParseError::BadRequest)?;
                content_length = Some(n);
            }
            "authorization" => {
                let v = value.trim();
                if let Some(rest) = v
                    .strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("bearer "))
                {
                    bearer = Some(rest.trim().to_string());
                }
            }
            "accept" => accept = Some(value.trim().to_string()),
            "content-type" => content_type = Some(value.trim().to_string()),
            "connection" => connection = Some(value.trim().to_ascii_lowercase()),
            _ => {}
        }
    }

    let body_len = content_length.unwrap_or(0);
    if body_len > MAX_BODY_BYTES {
        return Err(ParseError::BodyTooLarge {
            got: body_len,
            max: MAX_BODY_BYTES,
        });
    }
    let body_start = header_end;
    let body_end = body_start.checked_add(body_len).ok_or(ParseError::BadRequest)?;
    if raw.len() < body_end {
        return Err(ParseError::Incomplete);
    }
    let body = raw[body_start..body_end].to_vec();

    let keep_alive = match connection.as_deref() {
        Some("close") => false,
        Some("keep-alive") => true,
        _ => http_version != "HTTP/1.0",
    };

    Ok(ParsedRequest {
        method,
        path,
        body,
        bearer,
        accept,
        content_type,
        keep_alive,
    })
}

/// How many bytes of `raw` this request consumed (headers + body).
pub fn request_span(raw: &[u8]) -> Result<usize, ParseError> {
    let header_end = find_header_end(raw).ok_or(ParseError::Incomplete)?;
    let header_text = std::str::from_utf8(&raw[..header_end]).map_err(|_| ParseError::BadRequest)?;
    let mut content_length = 0usize;
    for line in header_text.split("\r\n").skip(1) {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = split_header(line) {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().map_err(|_| ParseError::BadRequest)?;
            }
        }
    }
    if content_length > MAX_BODY_BYTES {
        return Err(ParseError::BodyTooLarge {
            got: content_length,
            max: MAX_BODY_BYTES,
        });
    }
    Ok(header_end + content_length)
}

fn find_header_end(raw: &[u8]) -> Option<usize> {
    raw.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

fn parse_request_line(line: &str) -> Result<(String, &str), ParseError> {
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(ParseError::BadRequest)?.to_string();
    let target = parts.next().ok_or(ParseError::BadRequest)?;
    Ok((method, target))
}

fn strip_query(target: &str) -> &str {
    target.split('?').next().unwrap_or("/")
}

fn split_header(line: &str) -> Option<(&str, &str)> {
    let (n, v) = line.split_once(':')?;
    Some((n, v))
}

/// Parser failures mapped onto HTTP statuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    BadRequest,
    HeadersTooLarge,
    BodyTooLarge { got: usize, max: usize },
}

impl ParseError {
    pub fn to_io(&self) -> io::Error {
        match self {
            ParseError::Incomplete => io::Error::new(ErrorKind::UnexpectedEof, "incomplete HTTP"),
            ParseError::BadRequest => io::Error::new(ErrorKind::InvalidData, "bad HTTP request"),
            ParseError::HeadersTooLarge => {
                io::Error::new(ErrorKind::InvalidData, "HTTP headers too large")
            }
            ParseError::BodyTooLarge { .. } => {
                io::Error::new(ErrorKind::InvalidData, "HTTP body too large")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_with_query_and_accept() {
        let raw = b"GET /lean/v0/health?x=1 HTTP/1.1\r\nAccept: application/json\r\nHost: x\r\n\r\n";
        let p = parse_http_request(raw).unwrap();
        assert_eq!(p.method, "GET");
        assert_eq!(p.path, "/lean/v0/health");
        assert_eq!(p.accept.as_deref(), Some("application/json"));
        assert!(p.body.is_empty());
        assert!(p.keep_alive);
    }

    #[test]
    fn parses_content_length_body() {
        let raw = b"POST /lean/v0/admin/aggregator HTTP/1.1\r\nContent-Length: 16\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"enabled\":true}";
        let p = parse_http_request(raw).unwrap();
        assert_eq!(p.body, br#"{"enabled":true}"#);
        assert!(!p.keep_alive);
        assert_eq!(request_span(raw).unwrap(), raw.len());
    }

    #[test]
    fn incomplete_until_body_arrives() {
        let raw = b"POST /x HTTP/1.1\r\nContent-Length: 4\r\n\r\nab";
        assert_eq!(parse_http_request(raw), Err(ParseError::Incomplete));
    }

    #[test]
    fn parses_bearer() {
        let raw = b"GET /lean/v1/health HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n";
        let p = parse_http_request(raw).unwrap();
        assert_eq!(p.bearer.as_deref(), Some("secret"));
    }
}
