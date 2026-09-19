//! Lean P2P surface for the node — re-exports `ethean-network`.
//!
//! Legacy Beacon/libp2p modules under this directory are not compiled.

pub use ethean_network::{
    admit, decode_gossip, delta_for, dial_quic, dial_quic_pending, encode_gossip, handle_status,
    parse_quic_udp, prepare_transport, probe_udp_status, reject_non_quic, validate_gossip_payload,
    BoundTransport, GossipAction, GossipIngress, LeanGossipTopics, NetworkError, NodeIdentity,
    PeerManager, PeerRecord, PumpEvent, QuicUdpAddr, RequestTracker, Result, StatusExchange,
    SwarmFacade, TransportConfig, UdpDialProbe, MAX_INBOUND_PEERS, MAX_OUTBOUND_PEERS,
    MAX_PEERS_PER_IP, SCORE_ACCEPT, SCORE_IGNORE, SCORE_REJECT,
};

#[cfg(feature = "libp2p-quic")]
pub use ethean_network::QuicSwarm;
