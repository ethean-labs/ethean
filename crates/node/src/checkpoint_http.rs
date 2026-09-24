//! Minimal blocking HTTP/1.1 GET for checkpoint sync (plain `http://` only).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Parsed `http://host[:port]/path` target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpTarget {
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// Split a plain-HTTP URL into host, port and path.
pub fn parse_http_url(url: &str) -> Result<HttpTarget, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or("checkpoint sync needs a plain http:// URL")?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if !h.contains(']') || h.ends_with(']') => {
            (h, p.parse::<u16>().map_err(|_| "invalid port")?)
        }
        _ => (authority, 80),
    };
    if host.is_empty() {
        return Err("empty host".into());
    }
    Ok(HttpTarget {
        host: host.trim_matches(|c| c == '[' || c == ']').to_string(),
        port,
        path: path.to_string(),
    })
}

/// GET `url` with `Accept: application/octet-stream`; returns status and body.
pub fn http_get(url: &str, timeout: Duration) -> Result<(u16, Vec<u8>), String> {
    let target = parse_http_url(url)?;
    let addr = (target.host.as_str(), target.port)
        .to_socket_addrs()
        .map_err(|e| format!("resolve {}: {e}", target.host))?
        .next()
        .ok_or("host did not resolve")?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/octet-stream\r\nConnection: close\r\n\r\n",
        target.path, target.host
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let header_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("malformed HTTP response")?;
    let head = String::from_utf8_lossy(&raw[..header_end]);
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or("missing HTTP status")?;
    let mut body = raw[header_end + 4..].to_vec();
    let content_length = head.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.eq_ignore_ascii_case("content-length")
            .then(|| v.trim().parse::<usize>().ok())
            .flatten()
    });
    if let Some(n) = content_length {
        if body.len() < n {
            return Err(format!("truncated body: {} of {n} bytes", body.len()));
        }
        body.truncate(n);
    }
    Ok((status, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_port_and_path() {
        let t = parse_http_url("http://10.0.0.7:5052/lean/v0/states/finalized").unwrap();
        assert_eq!(
            (t.host.as_str(), t.port, t.path.as_str()),
            ("10.0.0.7", 5052, "/lean/v0/states/finalized")
        );
        let t = parse_http_url("http://helper/x").unwrap();
        assert_eq!(t.port, 80);
        assert!(parse_http_url("https://x/y").is_err());
        assert!(parse_http_url("http://:1/").is_err());
    }
}
