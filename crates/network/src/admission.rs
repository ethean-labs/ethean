//! Peer admission limits (static ENR / dial policy scaffolding).

/// Maximum inbound connections.
pub const MAX_INBOUND_PEERS: usize = 50;

/// Maximum outbound / dialed peers.
pub const MAX_OUTBOUND_PEERS: usize = 50;

/// Maximum peers sharing one IP.
pub const MAX_PEERS_PER_IP: usize = 5;

/// Decide whether a new peer may be admitted.
pub fn admit(inbound: bool, current_inbound: usize, current_outbound: usize, same_ip: usize) -> bool {
    if same_ip >= MAX_PEERS_PER_IP {
        return false;
    }
    if inbound {
        current_inbound < MAX_INBOUND_PEERS
    } else {
        current_outbound < MAX_OUTBOUND_PEERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ip_flood() {
        assert!(!admit(true, 0, 0, MAX_PEERS_PER_IP));
    }
}
