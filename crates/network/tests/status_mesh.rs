//! Two-node QuicSwarm Status dial smoke (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use ethean_network::gossip::PumpEvent;
use ethean_network::{QuicSwarm, TransportConfig};
use ethean_network_wire::Status;
use ethean_primitives::Hash32;
use libp2p::Multiaddr;
use sha2::{Digest, Sha256};
use std::time::Duration;

fn sample_status(fork: &str, head_slot: u64) -> Status {
    Status {
        genesis_root: [1u8; 32],
        fork_segment: fork.into(),
        head_slot,
        head_root: [head_slot as u8; 32],
        finalized_slot: 0,
        finalized_root: [0u8; 32],
    }
}

fn fingerprint(peer_bytes: &[u8]) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(peer_bytes);
    let dig = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&dig);
    out
}

fn loopback_with_peer(listen: &Multiaddr, peer_id: &str) -> String {
    let s = listen.to_string();
    let port = s
        .split('/')
        .skip_while(|p| *p != "udp")
        .nth(1)
        .expect("udp port");
    format!("/ip4/127.0.0.1/udp/{port}/quic-v1/p2p/{peer_id}")
}

#[tokio::test]
async fn two_nodes_dial_and_status_response() {
    let fork = "aabbccdd";
    let cfg = TransportConfig {
        listen_port: 0,
        idle_timeout_ms: 5_000,
    };
    let mut a = QuicSwarm::bind_for_fork_segment(&cfg, fork)
        .await
        .expect("bind a");
    let mut b = QuicSwarm::bind_for_fork_segment(&cfg, fork)
        .await
        .expect("bind b");

    let local_a = sample_status(fork, 1);
    let local_b = sample_status(fork, 7);
    a.set_local_status_bytes(local_a.encode().expect("enc a"));
    b.set_local_status_bytes(local_b.encode().expect("enc b"));

    let dial = loopback_with_peer(&a.listen_addr, &a.peer_id.to_string());
    eprintln!("dialing {dial} from {}", b.peer_id);
    b.dial(&dial).expect("dial a");

    let mut b_connected = false;
    let mut a_connected = false;
    let mut seen_b = Vec::new();
    let deadline = Duration::from_secs(5);
    let start = std::time::Instant::now();
    while start.elapsed() < deadline && !(a_connected && b_connected) {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(30), a.pump_once()).await {
            seen_b.push(format!("a:{}", ev.kind_label()));
            if matches!(ev, PumpEvent::ConnectionEstablished { .. }) {
                a_connected = true;
            }
        }
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(30), b.pump_once()).await {
            seen_b.push(format!("b:{}", ev.kind_label()));
            if matches!(ev, PumpEvent::ConnectionEstablished { .. }) {
                b_connected = true;
            }
        }
    }
    assert!(
        a_connected && b_connected,
        "both sides should connect; events={seen_b:?}"
    );

    let peer_a = fingerprint(&a.peer_id.to_bytes());
    b.send_status_request(peer_a, local_b.encode().expect("enc b req"))
        .expect("send status");

    let mut got_response = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        let _ = tokio::time::timeout(Duration::from_millis(20), a.pump_once()).await;
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(50), b.pump_once()).await {
            if let PumpEvent::StatusResponse { peer, payload } = ev {
                assert_eq!(peer, peer_a);
                let remote = Status::decode(&payload).expect("decode status");
                assert_eq!(remote.head_slot, 1);
                assert_eq!(remote.fork_segment, fork);
                got_response = true;
                break;
            }
        }
    }
    assert!(got_response, "B should receive StatusResponse from A");
}
