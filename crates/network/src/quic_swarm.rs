//! Optional libp2p QUIC-v1 swarm with Lean gossipsub + Status req/resp.

#![cfg(feature = "libp2p-quic")]

use crate::error::NetworkError;
use crate::gossip::{LeanGossipTopics, PumpEvent, SMOKE_ATTESTATION_SUBNETS};
use crate::multiaddr::parse_quic_udp;
use crate::quic_blocks_codec::{blocks_by_root_behaviour, BlocksByRootCodec};
use crate::quic_range_codec::{blocks_by_range_behaviour, BlocksByRangeCodec};
use crate::quic_status_codec::{status_behaviour, StatusCodec};
use crate::quic_swarm_bind::{build_gossipsub, subscribe_all, wait_quic_listen};
use crate::transport::TransportConfig;
use ethean_primitives::Hash32;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::request_response;
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, ping, Multiaddr, PeerId, SwarmBuilder};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

type NetResult<T> = std::result::Result<T, NetworkError>;

/// Ping + gossipsub + Lean Status / blocks-by-root / blocks-by-range request_response.
#[derive(libp2p::swarm::NetworkBehaviour)]
pub(crate) struct LeanBehaviour {
    pub(crate) ping: ping::Behaviour,
    pub(crate) gossipsub: gossipsub::Behaviour,
    pub(crate) status: request_response::Behaviour<StatusCodec>,
    pub(crate) blocks_by_root: request_response::Behaviour<BlocksByRootCodec>,
    pub(crate) blocks_by_range: request_response::Behaviour<BlocksByRangeCodec>,
}

/// libp2p swarm listening on QUIC-v1 (UDP).
pub struct QuicSwarm {
    /// Local peer id.
    pub peer_id: PeerId,
    /// First QUIC listen multiaddr observed.
    pub listen_addr: Multiaddr,
    /// Subscribed Lean mesh topics (if any).
    pub topics: Option<LeanGossipTopics>,
    pub(crate) swarm: libp2p::Swarm<LeanBehaviour>,
    pub(crate) seen_ids: HashSet<Hash32>,
    /// Fingerprint → PeerId for Status outbound sends.
    pub(crate) peers: HashMap<Hash32, PeerId>,
    /// Local Status SSZ used to answer inbound Status requests.
    pub(crate) local_status: Option<Vec<u8>>,
    /// Signed-block bytes keyed by root for inbound blocks-by-root replies.
    pub(crate) blocks_by_root: HashMap<Hash32, Vec<u8>>,
    /// Signed-block bytes keyed by slot for inbound blocks-by-range replies.
    pub(crate) blocks_by_slot: HashMap<u64, Vec<u8>>,
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
        Self::bind_for_fork_segment_subnets(cfg, fork_segment, SMOKE_ATTESTATION_SUBNETS).await
    }

    /// Bind with an explicit attestation subnet subscription count.
    pub async fn bind_for_fork_segment_subnets(
        cfg: &TransportConfig,
        fork_segment: &str,
        attestation_subnets: u16,
    ) -> NetResult<Self> {
        let topics =
            LeanGossipTopics::from_fork_segment_subnets(fork_segment, attestation_subnets)?;
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
                status: status_behaviour(),
                blocks_by_root: blocks_by_root_behaviour(),
                blocks_by_range: blocks_by_range_behaviour(),
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
            peers: HashMap::new(),
            local_status: None,
            blocks_by_root: HashMap::new(),
            blocks_by_slot: HashMap::new(),
        })
    }

    /// Cache local Status SSZ for inbound Status replies.
    pub fn set_local_status_bytes(&mut self, bytes: Vec<u8>) {
        self.local_status.replace(bytes);
    }

    /// Insert or replace a block body served on inbound blocks-by-root.
    pub fn put_block_bytes(&mut self, root: Hash32, bytes: Vec<u8>) {
        self.blocks_by_root.insert(root, bytes);
    }

    /// Index a block body by slot for inbound blocks-by-range replies.
    pub fn put_block_at_slot(&mut self, slot: u64, root: Hash32, bytes: Vec<u8>) {
        self.blocks_by_root.insert(root, bytes.clone());
        self.blocks_by_slot.insert(slot, bytes);
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

    /// Send a Status request to a connected peer fingerprint.
    pub fn send_status_request(&mut self, peer: Hash32, payload: Vec<u8>) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for Status request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .status
            .send_request(&peer_id, payload);
        Ok(())
    }

    /// Send a blocks-by-root request to a connected peer fingerprint.
    pub fn send_blocks_by_root_request(
        &mut self,
        peer: Hash32,
        payload: Vec<u8>,
    ) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for blocks-by-root request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_root
            .send_request(&peer_id, payload);
        Ok(())
    }

    /// Send a blocks-by-range request to a connected peer fingerprint.
    pub fn send_blocks_by_range_request(
        &mut self,
        peer: Hash32,
        payload: Vec<u8>,
    ) -> NetResult<()> {
        let Some(peer_id) = self.peers.get(&peer).copied() else {
            return Err(NetworkError::Handshake(
                "peer not connected for blocks-by-range request".into(),
            ));
        };
        let _ = self
            .swarm
            .behaviour_mut()
            .blocks_by_range
            .send_request(&peer_id, payload);
        Ok(())
    }

    /// Pump one swarm event; validates inbound gossip against Lean rules.
    pub async fn pump_once(&mut self) -> PumpEvent {
        match self.swarm.select_next_some().await {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                let peer = self.remember_peer(peer_id);
                PumpEvent::ConnectionEstablished { peer: Some(peer) }
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                let peer = self.forget_peer(&peer_id);
                PumpEvent::ConnectionClosed { peer: Some(peer) }
            }
            SwarmEvent::OutgoingConnectionError { .. } => PumpEvent::OutgoingError,
            SwarmEvent::IncomingConnectionError { .. } => PumpEvent::IncomingError,
            SwarmEvent::NewListenAddr { .. } => PumpEvent::NewListenAddr,
            SwarmEvent::Behaviour(LeanBehaviourEvent::Gossipsub(ev)) => {
                self.handle_gossip_event(ev)
            }
            SwarmEvent::Behaviour(LeanBehaviourEvent::Status(ev)) => self.handle_status_event(ev),
            SwarmEvent::Behaviour(LeanBehaviourEvent::BlocksByRoot(ev)) => {
                self.handle_blocks_by_root_event(ev)
            }
            SwarmEvent::Behaviour(LeanBehaviourEvent::BlocksByRange(ev)) => {
                self.handle_blocks_by_range_event(ev)
            }
            SwarmEvent::Behaviour(_) => PumpEvent::Behaviour,
            _ => PumpEvent::Other,
        }
    }
}
