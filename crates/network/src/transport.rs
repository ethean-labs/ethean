//! QUIC/UDP transport bind (swarm protocol still separate).

use crate::error::{NetworkError, Result};
use crate::identity::NodeIdentity;
use std::net::UdpSocket;

/// Desired listen configuration for QUIC-v1 / UDP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportConfig {
    /// UDP listen port (`0` = OS ephemeral, allowed for tests).
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

/// UDP socket held after a successful listen bind (pre-libp2p QUIC swarm).
#[derive(Debug)]
pub struct BoundTransport {
    /// Bound UDP socket (non-blocking).
    pub socket: UdpSocket,
    /// Local port actually bound.
    pub listen_port: u16,
    /// Node identity fingerprint used for this bind.
    pub identity: NodeIdentity,
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

/// Bind a UDP listen socket for QUIC-v1; does not start a libp2p swarm.
pub fn prepare_transport(
    identity: &NodeIdentity,
    cfg: &TransportConfig,
) -> Result<BoundTransport> {
    if identity.fingerprint == [0u8; 32] {
        return Err(NetworkError::Handshake(
            "empty node identity fingerprint refused".into(),
        ));
    }
    let listen = format!("/ip4/0.0.0.0/udp/{}/quic-v1", cfg.listen_port);
    reject_non_quic(&listen)?;

    let socket = UdpSocket::bind(("0.0.0.0", cfg.listen_port)).map_err(|e| {
        NetworkError::Handshake(format!("UDP bind failed on port {}: {e}", cfg.listen_port))
    })?;
    socket
        .set_nonblocking(true)
        .map_err(|e| NetworkError::Handshake(format!("set_nonblocking: {e}")))?;
    let listen_port = socket
        .local_addr()
        .map(|a| a.port())
        .map_err(|e| NetworkError::Handshake(format!("local_addr: {e}")))?;

    Ok(BoundTransport {
        socket,
        listen_port,
        identity: identity.clone(),
    })
}

/// Dial without a bound [`crate::QuicSwarm`].
///
/// For UDP path / Status reachability checks use [`crate::probe_udp_status`].
/// With `libp2p-quic`, call [`crate::SwarmFacade::bind_quic_swarm`] then
/// [`crate::SwarmFacade::dial_quic_peer`].
pub fn dial_quic(_bound: &BoundTransport, multiaddr: &str) -> Result<()> {
    crate::dial::dial_quic_pending(multiaddr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::NetworkError;

    #[test]
    fn rejects_tcp_and_ws() {
        assert!(reject_non_quic("/ip4/1.2.3.4/tcp/9000").is_err());
        assert!(reject_non_quic("/ip4/1.2.3.4/ws").is_err());
        assert!(reject_non_quic("/ip4/1.2.3.4/udp/9000/quic-v1").is_ok());
    }

    #[test]
    fn prepare_binds_ephemeral_udp() {
        let id = NodeIdentity::from_seed(b"ethean");
        let cfg = TransportConfig {
            listen_port: 0,
            idle_timeout_ms: 1_000,
        };
        let bound = prepare_transport(&id, &cfg).expect("bind");
        assert_ne!(bound.listen_port, 0);
        assert!(matches!(
            dial_quic(&bound, "/ip4/127.0.0.1/udp/1/quic-v1"),
            Err(NetworkError::TransportPending(_))
        ));
    }
}
