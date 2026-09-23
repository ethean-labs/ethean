//! Peer connection lifecycle events.

use ethean_metrics::lean::inc;

/// Peer connection lifecycle (swarm pump).
pub fn peer_connected(outbound: bool) {
    inc(
        "lean_peer_connection_events_total",
        &[direction(outbound), "success"],
        1.0,
    );
}

pub fn peer_connect_failed(outbound: bool) {
    inc(
        "lean_peer_connection_events_total",
        &[direction(outbound), "error"],
        1.0,
    );
}

pub fn peer_disconnected(outbound: bool, reason: &'static str) {
    inc(
        "lean_peer_disconnection_events_total",
        &[direction(outbound), reason],
        1.0,
    );
}

fn direction(outbound: bool) -> &'static str {
    if outbound {
        "outbound"
    } else {
        "inbound"
    }
}
