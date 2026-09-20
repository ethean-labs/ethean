//! Minimal Lean `/lean/v1` TCP HTTP listener (same style as metrics scrape).

use crate::auth::BindScope;
use crate::error::RpcError;
use crate::json_body::{duties_json, finalized_json, head_json, identity_json, sync_json};
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
    if let Err(e) = crate::auth::validate_admin_token(scope, state.admin_token()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string()));
    }
    let listener = TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    info!(%bound, "Lean HTTP listening (/lean/v1/…)");
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
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let (method, path, bearer) = parse_request(&req);
    let incoming = IncomingRequest {
        method: &method,
        path: &path,
        body_len: 0,
        bearer: bearer.as_deref(),
    };
    let (status, content_type, body) = match dispatch(&incoming, scope, state.admin_token()) {
        Ok(route) => respond(route, &state),
        Err(e) => error_response(e),
    };
    let resp = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        reason(status),
        body.len(),
    );
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn respond(route: Route, state: &SharedApiState) -> (u16, &'static str, String) {
    let snap = state.snapshot();
    match route {
        Route::Health => (200, "text/plain; charset=utf-8", "ok\n".into()),
        Route::Ready => {
            if state.is_ready() {
                (200, "text/plain; charset=utf-8", "ready\n".into())
            } else {
                (503, "text/plain; charset=utf-8", "not ready\n".into())
            }
        }
        Route::NodeIdentity => json_ok(identity_json(&snap)),
        Route::ChainHead => json_ok(head_json(&snap.head)),
        Route::ChainFinalized => json_ok(finalized_json(&snap.finalized)),
        Route::ChainSync => json_ok(sync_json(&snap.sync)),
        Route::ValidatorDuties => json_ok(duties_json()),
        Route::AdminShutdown => {
            state.request_shutdown();
            (200, "application/json", r#"{"ok":true}"#.into())
        }
        Route::AdminEvents => (
            501,
            "application/json",
            r#"{"error":"events stream not implemented"}"#.into(),
        ),
    }
}

fn json_ok(v: serde_json::Value) -> (u16, &'static str, String) {
    (200, "application/json", v.to_string())
}

fn error_response(err: RpcError) -> (u16, &'static str, String) {
    let status = match &err {
        RpcError::UnknownRoute(_) => 404,
        RpcError::BodyTooLarge { .. } | RpcError::BadRequest(_) => 400,
        RpcError::Unauthorized => 401,
        RpcError::Forbidden => 403,
        RpcError::Unavailable(_) => 503,
    };
    let body = serde_json::json!({ "error": err.to_string() }).to_string();
    (status, "application/json", body)
}

fn parse_request(req: &str) -> (String, String, Option<String>) {
    let mut lines = req.lines();
    let line = lines.next().unwrap_or("");
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let path = parts
        .next()
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let mut bearer = None;
    for l in lines {
        let lower = l.to_ascii_lowercase();
        if lower.starts_with("authorization:") {
            let v = l.split_once(':').map(|(_, r)| r.trim()).unwrap_or("");
            if let Some(rest) = v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")) {
                bearer = Some(rest.trim().to_string());
            }
        }
        if l.is_empty() {
            break;
        }
    }
    (method, path, bearer)
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
    use super::*;

    #[test]
    fn parses_bearer() {
        let raw = "GET /lean/v1/health HTTP/1.1\r\nAuthorization: Bearer secret\r\n\r\n";
        let (m, p, b) = parse_request(raw);
        assert_eq!(m, "GET");
        assert_eq!(p, "/lean/v1/health");
        assert_eq!(b.as_deref(), Some("secret"));
    }
}
