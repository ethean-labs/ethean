//! Lean HTTP/1.1 listener: request-line + headers, Content-Length bodies, keep-alive.

use crate::auth::BindScope;
use crate::handlers::{error_json, handle_route, HttpReply};
use crate::http_sse::{stream_admin_events, wants_sse};
use crate::limits::{MAX_BODY_BYTES, MAX_HEADER_BYTES};
use crate::parse::{parse_http_request, request_span, ParseError};
use crate::routes::Route;
use crate::server::{dispatch, IncomingRequest};
use crate::state::SharedApiState;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

/// Bind, spawn accept loop, return the actual bound address.
pub async fn spawn_lean_http(
    addr: SocketAddr,
    state: Arc<SharedApiState>,
) -> io::Result<SocketAddr> {
    let scope = if addr.ip().is_loopback() {
        BindScope::Loopback
    } else {
        BindScope::Public
    };
    let _ = crate::auth::validate_admin_token(scope, state.admin_token());
    let listener = TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    info!(%bound, "Lean HTTP listening (/lean/v0/…, /lean/v1 aliases)");
    tokio::spawn(async move {
        loop {
            let Ok((stream, peer)) = listener.accept().await else {
                break;
            };
            let state = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_conn(stream, state, scope).await {
                    warn!(%peer, error = %e, "lean HTTP connection error");
                }
            });
        }
    });
    Ok(bound)
}

async fn handle_conn(
    mut stream: TcpStream,
    state: Arc<SharedApiState>,
    scope: BindScope,
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(4096);
    loop {
        let req = match read_one_request(&mut stream, &mut buf).await? {
            Some(r) => r,
            None => return Ok(()),
        };
        let incoming = IncomingRequest {
            method: &req.method,
            path: &req.path,
            body_len: req.body.len(),
            bearer: req.bearer.as_deref(),
        };
        match dispatch(&incoming, scope, state.admin_token()) {
            Ok(Route::AdminEvents) if wants_sse(req.accept.as_deref()) => {
                stream_admin_events(&mut stream, &state).await?;
                break;
            }
            Ok(route) => {
                let reply = handle_route(route, &state, &req.body);
                write_reply(&mut stream, &reply, req.keep_alive).await?;
                if !req.keep_alive {
                    break;
                }
            }
            Err(e) => {
                let reply = error_json(e);
                write_reply(&mut stream, &reply, false).await?;
                break;
            }
        }
        buf.clear();
    }
    Ok(())
}

async fn read_one_request(
    stream: &mut TcpStream,
    buf: &mut Vec<u8>,
) -> io::Result<Option<crate::parse::ParsedRequest>> {
    loop {
        match parse_http_request(buf) {
            Ok(req) => {
                let span = request_span(buf).map_err(|e| e.to_io())?;
                buf.drain(..span);
                return Ok(Some(req));
            }
            Err(ParseError::Incomplete) => {}
            Err(ParseError::BodyTooLarge { got, max }) => {
                let reply = error_json(crate::error::RpcError::BodyTooLarge { got, max });
                write_reply(stream, &reply, false).await?;
                return Ok(None);
            }
            Err(e) => {
                let reply = error_json(crate::error::RpcError::BadRequest(format!("{e:?}")));
                write_reply(stream, &reply, false).await?;
                return Ok(None);
            }
        }
        if buf.len() > MAX_HEADER_BYTES + MAX_BODY_BYTES {
            let reply = error_json(crate::error::RpcError::BodyTooLarge {
                got: buf.len(),
                max: MAX_BODY_BYTES,
            });
            write_reply(stream, &reply, false).await?;
            return Ok(None);
        }
        let mut chunk = [0u8; 8192];
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

async fn write_reply(stream: &mut TcpStream, reply: &HttpReply, keep_alive: bool) -> io::Result<()> {
    let conn = if keep_alive { "keep-alive" } else { "close" };
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: {conn}\r\n\r\n",
        reply.status,
        reason(reply.status),
        reply.content_type,
        reply.body.len(),
    );
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(&reply.body).await?;
    if !keep_alive {
        stream.shutdown().await.ok();
    }
    Ok(())
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::parse_http_request;

    #[test]
    fn parse_still_used_by_listener() {
        let raw = b"GET /lean/v0/health HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n";
        let p = parse_http_request(raw).unwrap();
        assert_eq!(p.path, "/lean/v0/health");
        assert_eq!(p.bearer.as_deref(), Some("secret"));
    }
}
