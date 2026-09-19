//! Lean Consensus P2P networking: admission, gossip validation, req/resp, QUIC facade.

#![forbid(unsafe_code)]

pub mod admission;
pub mod dial;
pub mod error;
pub mod gossip;
pub mod identity;
pub mod multiaddr;
pub mod peer_manager;
pub mod quic_swarm;
#[cfg(feature = "libp2p-quic")]
mod quic_events;
#[cfg(feature = "libp2p-quic")]
mod quic_framed;
#[cfg(feature = "libp2p-quic")]
pub mod quic_blocks_codec;
#[cfg(feature = "libp2p-quic")]
pub mod quic_status_codec;
pub mod reqresp;
pub mod swarm;
pub mod transport;

pub use admission::{admit, MAX_INBOUND_PEERS, MAX_OUTBOUND_PEERS, MAX_PEERS_PER_IP};
pub use dial::{dial_quic_pending, probe_udp_status, UdpDialProbe};
pub use error::{NetworkError, Result};
pub use gossip::{
    decode_gossip, delta_for, encode_gossip, validate_gossip_payload, GossipAction, GossipIngress,
    LeanGossipTopics, PumpEvent, SCORE_ACCEPT, SCORE_IGNORE, SCORE_REJECT,
};
pub use identity::NodeIdentity;
pub use multiaddr::{parse_quic_udp, QuicUdpAddr};
pub use peer_manager::{PeerManager, PeerRecord};
#[cfg(feature = "libp2p-quic")]
pub use quic_swarm::QuicSwarm;
pub use reqresp::{
    blocks_by_root_for_roots, blocks_by_root_for_status_gap, blocks_by_root_protocol_id,
    decode_blocks_by_root_response, encode_blocks_by_root, encode_blocks_by_root_response,
    handle_status, prepare_blocks_by_root_for_roots, prepare_blocks_by_root_outbound,
    prepare_status_outbounds, OutboundBlocksByRootRequest, OutboundStatusRequest, RequestId,
    RequestTracker, StatusExchange, StatusSessionBook,
};
pub use swarm::SwarmFacade;
pub use transport::{
    dial_quic, prepare_transport, reject_non_quic, BoundTransport, TransportConfig,
};
