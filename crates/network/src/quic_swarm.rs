//! Optional libp2p QUIC-v1 swarm (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::multiaddr::parse_quic_udp;
use crate::transport::TransportConfig;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, ping, Multiaddr, PeerId, SwarmBuilder};
use std::time::Duration;

type NetResult<T> = std::result::Result<T, NetworkError>;

/// Ping-only behaviour; QUIC provides security and multiplexing.
#[derive(libp2p::swarm::NetworkBehaviour)]
struct LeanBehaviour {
    ping: ping::Behaviour,
}

/// libp2p swarm listening on QUIC-v1 (UDP).
pub struct QuicSwarm {
    /// Local peer id.
    pub peer_id: PeerId,
    /// First QUIC listen multiaddr observed.
    pub listen_addr: Multiaddr,
    swarm: libp2p::Swarm<LeanBehaviour>,
}

impl std::fmt::Debug for QuicSwarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicSwarm")
            .field("peer_id", &self.peer_id)
            .field("listen_addr", &self.listen_addr)
            .finish_non_exhaustive()
    }
}

impl QuicSwarm {
    /// Bind a QUIC-v1 listener (no TCP/WS listen).
    pub async fn bind(cfg: &TransportConfig) -> NetResult<Self> {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic()
            .with_behaviour(|_| LeanBehaviour {
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(15)),
                ),
            })
            .map_err(|e| NetworkError::Handshake(format!("behaviour: {e}")))?
            .build();

        let listen: Multiaddr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", cfg.listen_port)
            .parse()
            .map_err(|e| NetworkError::Handshake(format!("listen multiaddr: {e}")))?;
        swarm
            .listen_on(listen)
            .map_err(|e| NetworkError::Handshake(format!("listen_on: {e}")))?;

        let listen_addr = wait_quic_listen(&mut swarm).await?;
        crate::transport::reject_non_quic(&listen_addr.to_string())?;

        Ok(Self {
            peer_id,
            listen_addr,
            swarm,
        })
    }

    /// Dial `/ip4/.../udp/.../quic-v1` only.
    pub fn dial(&mut self, multiaddr: &str) -> NetResult<()> {
        crate::transport::reject_non_quic(multiaddr)?;
        let _ = parse_quic_udp(multiaddr)?;
        let addr: Multiaddr = multiaddr
            .parse()
            .map_err(|e| NetworkError::Handshake(format!("dial multiaddr: {e}")))?;
        self.swarm
            .dial(addr)
            .map_err(|e| NetworkError::Handshake(format!("swarm dial: {e}")))
    }

    /// Pump one swarm event for gossip / duty loops (opaque kind label).
    pub async fn pump_once(&mut self) -> &'static str {
        match self.swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { .. } => "connection_established",
            SwarmEvent::ConnectionClosed { .. } => "connection_closed",
            SwarmEvent::OutgoingConnectionError { .. } => "outgoing_error",
            SwarmEvent::IncomingConnectionError { .. } => "incoming_error",
            SwarmEvent::NewListenAddr { .. } => "new_listen_addr",
            SwarmEvent::Behaviour(_) => "behaviour",
            _ => "other",
        }
    }
}

async fn wait_quic_listen(
    swarm: &mut libp2p::Swarm<LeanBehaviour>,
) -> NetResult<Multiaddr> {
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                if address.to_string().contains("quic") {
                    return Ok(address);
                }
            }
            SwarmEvent::ListenerError { error, .. } => {
                return Err(NetworkError::Handshake(format!("listener: {error}")));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn binds_ephemeral_quic() {
        let mut swarm = QuicSwarm::bind(&TransportConfig {
            listen_port: 0,
            idle_timeout_ms: 1_000,
        })
        .await
        .expect("quic bind");
        assert!(swarm.listen_addr.to_string().contains("quic"));
        assert!(swarm.dial("/ip4/127.0.0.1/tcp/1").is_err());
    }
}
