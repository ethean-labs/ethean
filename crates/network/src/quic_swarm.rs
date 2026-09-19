//! Optional libp2p QUIC-v1 swarm with Lean gossipsub mesh (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::gossip::{GossipAction, GossipIngress, LeanGossipTopics, PumpEvent};
use crate::multiaddr::parse_quic_udp;
use crate::transport::TransportConfig;
use ethean_primitives::Hash32;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, ping, Multiaddr, PeerId, SwarmBuilder};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::time::Duration;

type NetResult<T> = std::result::Result<T, NetworkError>;

/// Ping + gossipsub; QUIC provides security and multiplexing.
#[derive(libp2p::swarm::NetworkBehaviour)]
struct LeanBehaviour {
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

/// libp2p swarm listening on QUIC-v1 (UDP).
pub struct QuicSwarm {
    /// Local peer id.
    pub peer_id: PeerId,
    /// First QUIC listen multiaddr observed.
    pub listen_addr: Multiaddr,
    /// Subscribed Lean mesh topics (if any).
    pub topics: Option<LeanGossipTopics>,
    swarm: libp2p::Swarm<LeanBehaviour>,
    seen_ids: HashSet<ethean_primitives::Hash32>,
}

impl std::fmt::Debug for QuicSwarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicSwarm")
            .field("peer_id", &self.peer_id)
            .field("listen_addr", &self.listen_addr)
            .field("topics", &self.topics)
            .finish_non_exhaustive()
    }
}

impl QuicSwarm {
    /// Bind a QUIC-v1 listener without gossip subscriptions.
    pub async fn bind(cfg: &TransportConfig) -> NetResult<Self> {
        Self::bind_inner(cfg, None).await
    }

    /// Bind and subscribe to Lean gossip topics for `fork_name`.
    pub async fn bind_for_fork(cfg: &TransportConfig, fork_name: &str) -> NetResult<Self> {
        let topics = LeanGossipTopics::from_fork_name(fork_name)?;
        Self::bind_inner(cfg, Some(topics)).await
    }

    /// Bind using an already-resolved fork segment (operator digest hex).
    pub async fn bind_for_fork_segment(
        cfg: &TransportConfig,
        fork_segment: &str,
    ) -> NetResult<Self> {
        let topics = LeanGossipTopics::from_fork_segment(fork_segment)?;
        Self::bind_inner(cfg, Some(topics)).await
    }

    async fn bind_inner(cfg: &TransportConfig, topics: Option<LeanGossipTopics>) -> NetResult<Self> {
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();
        let gossipsub = build_gossipsub(&keypair)?;

        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic()
            .with_behaviour(|_| LeanBehaviour {
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(15)),
                ),
                gossipsub,
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

        if let Some(ref t) = topics {
            subscribe_all(&mut swarm, t)?;
        }

        Ok(Self {
            peer_id,
            listen_addr,
            topics,
            swarm,
            seen_ids: HashSet::new(),
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

    /// Publish raw Snappy gossip bytes on a subscribed Lean topic.
    pub fn publish_gossip(&mut self, topic: &str, compressed: &[u8]) -> NetResult<()> {
        LeanGossipTopics::reject_if_eth2(topic)?;
        let t = IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(t, compressed.to_vec())
            .map_err(|e| NetworkError::Handshake(format!("gossip publish: {e}")))?;
        Ok(())
    }

    /// Pump one swarm event; validates inbound gossip against Lean rules.
    pub async fn pump_once(&mut self) -> PumpEvent {
        match self.swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { .. } => PumpEvent::ConnectionEstablished,
            SwarmEvent::ConnectionClosed { .. } => PumpEvent::ConnectionClosed,
            SwarmEvent::OutgoingConnectionError { .. } => PumpEvent::OutgoingError,
            SwarmEvent::IncomingConnectionError { .. } => PumpEvent::IncomingError,
            SwarmEvent::NewListenAddr { .. } => PumpEvent::NewListenAddr,
            SwarmEvent::Behaviour(LeanBehaviourEvent::Gossipsub(ev)) => {
                self.handle_gossip_event(ev)
            }
            SwarmEvent::Behaviour(_) => PumpEvent::Behaviour,
            _ => PumpEvent::Other,
        }
    }

    fn handle_gossip_event(&mut self, ev: gossipsub::Event) -> PumpEvent {
        match ev {
            gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            } => {
                let topic = message.topic.to_string();
                let peer = Some(peer_fingerprint(&propagation_source));
                let (action, plain) =
                    crate::gossip::validate_gossip_payload(&topic, &message.data, &mut self.seen_ids);
                let plain = match action {
                    GossipAction::Accept => plain,
                    _ => None,
                };
                PumpEvent::Gossip(GossipIngress {
                    action,
                    topic,
                    peer,
                    plain,
                })
            }
            gossipsub::Event::Subscribed { .. } => PumpEvent::GossipSubscribed,
            gossipsub::Event::Unsubscribed { .. } => PumpEvent::GossipUnsubscribed,
            _ => PumpEvent::Other,
        }
    }
}

fn peer_fingerprint(peer: &PeerId) -> Hash32 {
    let mut hasher = Sha256::new();
    hasher.update(peer.to_bytes());
    let dig = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&dig);
    out
}

fn build_gossipsub(keypair: &identity::Keypair) -> NetResult<gossipsub::Behaviour> {
    let config = gossipsub::ConfigBuilder::default()
        .validation_mode(ValidationMode::Permissive)
        .build()
        .map_err(|e| NetworkError::Handshake(format!("gossipsub config: {e}")))?;
    gossipsub::Behaviour::new(MessageAuthenticity::Signed(keypair.clone()), config)
        .map_err(|e| NetworkError::Handshake(format!("gossipsub behaviour: {e}")))
}

fn subscribe_all(
    swarm: &mut libp2p::Swarm<LeanBehaviour>,
    topics: &LeanGossipTopics,
) -> NetResult<()> {
    for topic in topics.as_slice() {
        LeanGossipTopics::reject_if_eth2(topic)?;
        let t = IdentTopic::new(topic);
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&t)
            .map_err(|e| NetworkError::Handshake(format!("subscribe {topic}: {e}")))?;
    }
    Ok(())
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

    #[tokio::test]
    async fn binds_and_subscribes_lean_topics() {
        let mut swarm = QuicSwarm::bind_for_fork(
            &TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 1_000,
            },
            "lstar",
        )
        .await
        .expect("quic+gossip");
        let topics = swarm.topics.as_ref().expect("topics");
        assert!(topics.block.contains("/leanconsensus/"));
        assert!(swarm
            .publish_gossip("/eth2/beacon_block/ssz_snappy", b"x")
            .is_err());
    }
}
