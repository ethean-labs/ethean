//! Minimal Prometheus scrape HTTP (`/metrics`, `/healthz`, `/readyz`).

use crate::export::export_prometheus_text;
use crate::shared::SharedRegistry;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};

/// Bind, spawn accept loop, return the actual bound address.
pub async fn spawn_metrics_server(
    addr: SocketAddr,
    registry: SharedRegistry,
    readiness: Arc<AtomicBool>,
) -> io::Result<SocketAddr> {
    let listener = TcpListener::bind(addr).await?;
    let bound = listener.local_addr()?;
    info!(%bound, "metrics HTTP listening (/metrics /healthz /readyz)");
    tokio::spawn(async move {
        loop {
            let Ok((stream, peer)) = listener.accept().await else {
                break;
            };
            let registry = registry.clone();
            let readiness = readiness.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_conn(stream, registry, readiness).await {
                    warn!(%peer, error = %e, "metrics connection error");
                }
            });
        }
    });
    Ok(bound)
}

async fn handle_conn(
    mut stream: TcpStream,
    registry: SharedRegistry,
    readiness: Arc<AtomicBool>,
) -> io::Result<()> {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = parse_path(&req);
    let (status, content_type, body) = match path.as_str() {
        "/metrics" => {
            let mut text = registry.with_ref(export_prometheus_text);
            text.push_str(&crate::lean::export_lean_text());
            (200, "text/plain; version=0.0.4; charset=utf-8", text)
        }
        "/healthz" => (200, "text/plain; charset=utf-8", "ok\n".into()),
        "/readyz" => {
            if readiness.load(Ordering::Relaxed) {
                (200, "text/plain; charset=utf-8", "ready\n".into())
            } else {
                (503, "text/plain; charset=utf-8", "not ready\n".into())
            }
        }
        _ => (404, "text/plain; charset=utf-8", "not found\n".into()),
    };
    let resp = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        reason(status),
        body.len(),
    );
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}

fn parse_path(req: &str) -> String {
    let line = req.lines().next().unwrap_or("");
    let mut parts = line.split_whitespace();
    let _method = parts.next();
    let path = parts.next().unwrap_or("/");
    path.split('?').next().unwrap_or("/").to_string()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        404 => "Not Found",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_path() {
        assert_eq!(
            parse_path("GET /metrics HTTP/1.1\r\nHost: x\r\n\r\n"),
            "/metrics"
        );
        assert_eq!(parse_path("GET /readyz?x=1 HTTP/1.1\r\n\r\n"), "/readyz");
    }
}
