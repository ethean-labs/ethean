//! Lean P2P surface for the node — re-exports `ethean-network`.
//!
//! Legacy Beacon/libp2p modules under this directory are not compiled.

pub use ethean_network::{
    admit, decode_gossip, delta_for, dial_quic, encode_gossip, handle_status, prepare_transport,
    reject_non_quic, validate_gossip_payload, BoundTransport, GossipAction, NetworkError,
    NodeIdentity, PeerManager, PeerRecord, RequestTracker, Result, StatusExchange, SwarmFacade,
    TransportConfig, MAX_INBOUND_PEERS, MAX_OUTBOUND_PEERS, MAX_PEERS_PER_IP, SCORE_ACCEPT,
    SCORE_IGNORE, SCORE_REJECT,
};
