//! QUIC transport facade (not yet bound to libp2p).

use crate::error::{NetworkError, Result};
use crate::identity::NodeIdentity;

/// Desired listen configuration for QUIC-v1 / UDP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportConfig {
    /// UDP listen port.
    pub listen_port: u16,
    /// Idle timeout milliseconds.
    pub idle_timeout_ms: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            listen_port: 9000,
            idle_timeout_ms: 30_000,
        }
    }
}

/// Refuse TCP / WebSocket multiaddrs; Lean networking is QUIC/UDP only.
pub fn reject_non_quic(multiaddr: &str) -> Result<()> {
    let lower = multiaddr.to_ascii_lowercase();
    if lower.contains("/tcp")
        || lower.contains("/ws")
        || lower.contains("websocket")
        || lower.contains("/wss")
    {
        return Err(NetworkError::Handshake(
            "non-QUIC transport refused (no TCP/WS fallback)".into(),
        ));
    }
    Ok(())
}

/// Build marker proving identity + config are present; real dial is Phase open gate.
pub fn prepare_transport(identity: &NodeIdentity, cfg: &TransportConfig) -> Result<()> {
    if identity.fingerprint == [0u8; 32] {
        return Err(NetworkError::Handshake(
            "empty node identity fingerprint refused".into(),
        ));
    }
    if cfg.listen_port == 0 {
        return Err(NetworkError::Handshake(
            "listen_port 0 refused".into(),
        ));
    }
    // Listen string check used by callers before dial.
    let listen = format!("/ip4/0.0.0.0/udp/{}/quic-v1", cfg.listen_port);
    reject_non_quic(&listen)?;
    Err(NetworkError::TransportPending(
        "libp2p QUIC-v1 swarm not wired; refuse simulated TCP/WS fallback",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tcp_and_ws() {
        assert!(reject_non_quic("/ip4/1.2.3.4/tcp/9000").is_err());
        assert!(reject_non_quic("/ip4/1.2.3.4/ws").is_err());
        assert!(reject_non_quic("/ip4/1.2.3.4/udp/9000/quic-v1").is_ok());
    }

    #[test]
    fn prepare_pending_after_checks() {
        let id = NodeIdentity::from_seed(b"ethean");
        let err = prepare_transport(&id, &TransportConfig::default()).unwrap_err();
        assert!(matches!(err, NetworkError::TransportPending(_)));
    }
}
