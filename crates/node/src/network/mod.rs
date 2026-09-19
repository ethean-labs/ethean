//! Lean P2P surface for the node — re-exports `ethean-network`.
//!
//! Legacy Beacon/libp2p modules under this directory are not compiled.

pub use ethean_network::{
    admit, blocks_by_root_for_roots, blocks_by_root_for_status_gap, blocks_by_root_protocol_id,
    decode_blocks_by_root_response, decode_gossip, delta_for, dial_quic, dial_quic_pending,
    encode_blocks_by_root, encode_blocks_by_root_response, encode_gossip, handle_status,
    parse_quic_udp, prepare_blocks_by_range_outbound, prepare_blocks_by_root_for_roots,
    prepare_blocks_by_root_outbound, prepare_status_outbounds, prepare_transport, probe_udp_status,
    reject_non_quic, validate_gossip_payload, BoundTransport, GossipAction, GossipIngress,
    LeanGossipTopics, NetworkError, NodeIdentity, OutboundBlocksByRangeRequest,
    OutboundBlocksByRootRequest, OutboundStatusRequest, PeerManager, PeerRecord, PumpEvent,
    QuicUdpAddr, RequestId, RequestTracker, Result, StatusExchange, StatusSessionBook, SwarmFacade,
    TransportConfig, UdpDialProbe, MAX_INBOUND_PEERS, MAX_OUTBOUND_PEERS, MAX_PEERS_PER_IP,
    SCORE_ACCEPT, SCORE_IGNORE, SCORE_REJECT,
};

#[cfg(feature = "libp2p-quic")]
pub use ethean_network::QuicSwarm;
