//! Long-lived Server-Sent Events stream for admin `/lean/v1/events`.

use crate::json_body::{admin_event_json, admin_event_name};
use crate::state::SharedApiState;
use std::io;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{interval, Instant};

/// True when the client asked for an event stream.
pub fn wants_sse(accept: Option<&str>) -> bool {
    accept
        .map(|a| a.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false)
}

/// Stream drained admin events until disconnect, shutdown, or idle timeout.
pub async fn stream_admin_events(
    stream: &mut TcpStream,
    state: &SharedApiState,
) -> io::Result<()> {
    let head = concat!(
        "HTTP/1.1 200 OK\r\n",
        "Content-Type: text/event-stream\r\n",
        "Cache-Control: no-cache\r\n",
        "Connection: keep-alive\r\n",
        "\r\n",
        ": connected\n\n",
    );
    stream.write_all(head.as_bytes()).await?;
    stream.flush().await?;

    let mut tick = interval(Duration::from_millis(500));
    let mut last_activity = Instant::now();
    let mut last_ping = Instant::now();
    let idle = Duration::from_secs(60);
    loop {
        if state.shutdown_requested() {
            let _ = stream.write_all(b"event: shutdown\ndata: {}\n\n").await;
            break;
        }
        let drained = state.drain_events(32);
        if !drained.is_empty() {
            for ev in &drained {
                let name = admin_event_name(ev);
                let data = admin_event_json(ev).to_string();
                let frame = format!("event: {name}\ndata: {data}\n\n");
                stream.write_all(frame.as_bytes()).await?;
            }
            stream.flush().await?;
            last_activity = Instant::now();
            last_ping = Instant::now();
        } else if last_activity.elapsed() >= idle {
            let _ = stream.write_all(b": idle timeout\n\n").await;
            break;
        } else if last_ping.elapsed() >= Duration::from_secs(15) {
            // Keep proxies from closing idle streams.
            stream.write_all(b": ping\n\n").await?;
            stream.flush().await?;
            last_ping = Instant::now();
        }
        tick.tick().await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_sse_accept() {
        assert!(wants_sse(Some("text/event-stream")));
        assert!(wants_sse(Some("text/event-stream, application/json")));
        assert!(!wants_sse(Some("application/json")));
        assert!(!wants_sse(None));
    }
}
