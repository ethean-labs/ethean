//! QUIC/UDP transport bind (swarm protocol still separate).

use crate::error::{NetworkError, Result};
use crate::identity::NodeIdentity;
use crate::node_key::NodeKey;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};

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

/// Listen address and libp2p identity used when binding the QUIC swarm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenIdentity {
    /// Interface to bind (`--socket-address`, default `0.0.0.0`).
    pub listen_ip: IpAddr,
    /// secp256k1 node key; `None` falls back to a per-run generated identity.
    pub node_key: Option<NodeKey>,
}

impl Default for ListenIdentity {
    fn default() -> Self {
        Self {
            listen_ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            node_key: None,
        }
    }
}

impl ListenIdentity {
    /// `/ip4/<ip>/udp/<port>/quic-v1` (or `/ip6/…`) listen multiaddr.
    pub fn listen_multiaddr(&self, port: u16) -> String {
        match self.listen_ip {
            IpAddr::V4(ip) => format!("/ip4/{ip}/udp/{port}/quic-v1"),
            IpAddr::V6(ip) => format!("/ip6/{ip}/udp/{port}/quic-v1"),
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

/// Bind a UDP listen socket for QUIC-v1 on all IPv4 interfaces.
pub fn prepare_transport(
    identity: &NodeIdentity,
    cfg: &TransportConfig,
) -> Result<BoundTransport> {
    prepare_transport_on(identity, cfg, IpAddr::V4(Ipv4Addr::UNSPECIFIED))
}

/// Bind a UDP listen socket for QUIC-v1 on `ip`; does not start a libp2p swarm.
pub fn prepare_transport_on(
    identity: &NodeIdentity,
    cfg: &TransportConfig,
    ip: IpAddr,
) -> Result<BoundTransport> {
    if identity.fingerprint == [0u8; 32] {
        return Err(NetworkError::Handshake(
            "empty node identity fingerprint refused".into(),
        ));
    }
    let listen = ListenIdentity {
        listen_ip: ip,
        node_key: None,
    }
    .listen_multiaddr(cfg.listen_port);
    reject_non_quic(&listen)?;

    let socket = UdpSocket::bind((ip, cfg.listen_port)).map_err(|e| {
        NetworkError::Handshake(format!(
            "UDP bind failed on {ip}:{}: {e}",
            cfg.listen_port
        ))
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
    fn listen_multiaddr_v4_and_v6() {
        let v4 = ListenIdentity::default();
        assert_eq!(v4.listen_multiaddr(9000), "/ip4/0.0.0.0/udp/9000/quic-v1");
        let v6 = ListenIdentity {
            listen_ip: "::1".parse().unwrap(),
            node_key: None,
        };
        assert_eq!(v6.listen_multiaddr(1), "/ip6/::1/udp/1/quic-v1");
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
