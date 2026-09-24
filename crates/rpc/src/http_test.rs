//! Integration: bind an ephemeral port and GET every `/lean/v0` hive route.

use crate::dto::{AggregatorStatusBody, CheckpointBody, ForkChoiceBody, HealthBody};
use crate::http::spawn_lean_http;
use crate::state::SharedApiState;
use crate::view::ForkChoiceView;
use ethean_primitives::HASH32_ZERO;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn http_get(addr: std::net::SocketAddr, path: &str) -> (u16, String, Vec<u8>) {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    s.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    split_http(&buf)
}

fn http_post(addr: std::net::SocketAddr, path: &str, body: &str) -> (u16, String, Vec<u8>) {
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    split_http(&buf)
}

fn split_http(raw: &[u8]) -> (u16, String, Vec<u8>) {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut ctype = String::new();
    for line in head.lines().skip(1) {
        if let Some(v) = line
            .to_ascii_lowercase()
            .strip_prefix("content-type:")
            .or_else(|| {
                if line.to_ascii_lowercase().starts_with("content-type:") {
                    Some("")
                } else {
                    None
                }
            })
        {
            let _ = v;
        }
        if line.to_ascii_lowercase().starts_with("content-type:") {
            ctype = line.split_once(':').unwrap().1.trim().to_string();
        }
    }
    (status, ctype, body.as_bytes().to_vec())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lean_v0_routes_on_ephemeral_port() {
    let state = SharedApiState::new("");
    let mut snap = state.snapshot();
    snap.fork_choice = ForkChoiceView::genesis(HASH32_ZERO, 4, b"STATE".to_vec(), b"BLOCK".to_vec());
    state.publish(snap);
    state.set_ready(true);

    let addr = spawn_lean_http("127.0.0.1:0".parse().unwrap(), state.clone())
        .await
        .unwrap();

    let (st, ctype, body) = http_get(addr, "/lean/v0/health");
    assert_eq!(st, 200);
    assert_eq!(ctype, "application/json");
    let health: HealthBody = serde_json::from_slice(&body).unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.service, "lean-rpc-api");

    let (st, _, body) = http_get(addr, "/lean/v1/health");
    assert_eq!(st, 200);
    let _: HealthBody = serde_json::from_slice(&body).unwrap();

    let (st, _, body) = http_get(addr, "/lean/v0/checkpoints/justified");
    assert_eq!(st, 200);
    let cp: CheckpointBody = serde_json::from_slice(&body).unwrap();
    assert_eq!(cp.slot, 0);
    assert_eq!(cp.root, HASH32_ZERO);

    let (st, _, body) = http_get(addr, "/lean/v0/fork_choice");
    assert_eq!(st, 200);
    let fc: ForkChoiceBody = serde_json::from_slice(&body).unwrap();
    assert_eq!(fc.nodes.len(), 1);
    assert_eq!(fc.head, HASH32_ZERO);
    assert_eq!(fc.justified.root, HASH32_ZERO);
    assert_eq!(fc.finalized.root, HASH32_ZERO);
    assert_eq!(fc.safe_target, HASH32_ZERO);
    assert!(fc.validator_count > 0);

    let (st, ctype, body) = http_get(addr, "/lean/v0/states/finalized");
    assert_eq!(st, 200);
    assert_eq!(ctype, "application/octet-stream");
    assert_eq!(body, b"STATE");

    let (st, ctype, body) = http_get(addr, "/lean/v0/blocks/finalized");
    assert_eq!(st, 200);
    assert_eq!(ctype, "application/octet-stream");
    assert_eq!(body, b"BLOCK");

    let (st, _, body) = http_get(addr, "/lean/v0/admin/aggregator");
    assert_eq!(st, 200);
    let ag: AggregatorStatusBody = serde_json::from_slice(&body).unwrap();
    assert!(!ag.is_aggregator);

    let (st, _, body) = http_post(addr, "/lean/v0/admin/aggregator", r#"{"enabled":true}"#);
    assert_eq!(st, 200);
    assert!(serde_json::from_slice::<serde_json::Value>(&body).unwrap()["is_aggregator"]
        .as_bool()
        .unwrap());

    let (st, _, _) = http_get(addr, "/eth/v1/node/health");
    assert_eq!(st, 404);

    let (st, _, _) = http_get(addr, "/lean/v0/health?x=1");
    assert_eq!(st, 200);
}

struct EchoDriver;

impl crate::test_driver::TestDriver for EchoDriver {
    fn fork_choice_init(&self, body: &[u8]) -> Result<(), String> {
        if body.starts_with(b"{") {
            Ok(())
        } else {
            Err("bad anchor".into())
        }
    }
    fn fork_choice_step(&self, body: &[u8]) -> Result<String, String> {
        Ok(format!(r#"{{"accepted":true,"error":null,"len":{}}}"#, body.len()))
    }
    fn state_transition(&self, _: &[u8]) -> Result<String, String> {
        Ok(r#"{"succeeded":true,"error":null,"post":null}"#.into())
    }
    fn verify_signatures(&self, _: &[u8]) -> Result<String, String> {
        Ok(r#"{"succeeded":false,"error":"x"}"#.into())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn driver_routes_answer_only_when_installed() {
    let state = SharedApiState::new("");
    let addr = spawn_lean_http("127.0.0.1:0".parse().unwrap(), state.clone())
        .await
        .unwrap();
    let (st, _, _) = http_post(addr, "/lean/v0/test_driver/fork_choice/init", "{}");
    assert_eq!(st, 404, "no driver installed");

    state.install_driver(std::sync::Arc::new(EchoDriver));
    let (st, _, body) = http_post(addr, "/lean/v0/test_driver/fork_choice/init", "{}");
    assert_eq!(st, 204);
    assert!(body.is_empty());
    let (st, _, _) = http_post(addr, "/lean/v0/test_driver/fork_choice/init", "nope");
    assert_eq!(st, 400);
    let big = format!(r#"{{"pad":"{}"}}"#, "x".repeat(200_000));
    let (st, ctype, body) = http_post(addr, "/lean/v0/test_driver/fork_choice/step", &big);
    assert_eq!(st, 200);
    assert_eq!(ctype, "application/json");
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["len"].as_u64().unwrap() as usize, big.len());
    let (st, _, _) = http_post(addr, "/lean/v0/test_driver/state_transition/run", "{}");
    assert_eq!(st, 200);
    let (st, _, _) = http_post(addr, "/lean/v0/test_driver/verify_signatures/run", "{}");
    assert_eq!(st, 200);
}
