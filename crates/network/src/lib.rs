//! Lean Consensus P2P networking: admission, gossip validation, req/resp, QUIC facade.

#![forbid(unsafe_code)]

pub mod admission;
pub mod dial;
pub mod error;
pub mod gossip;
pub mod identity;
pub mod multiaddr;
pub mod peer_manager;
pub mod reqresp;
pub mod swarm;
pub mod transport;

pub use admission::{admit, MAX_INBOUND_PEERS, MAX_OUTBOUND_PEERS, MAX_PEERS_PER_IP};
pub use dial::{dial_quic_pending, probe_udp_status, UdpDialProbe};
pub use error::{NetworkError, Result};
pub use gossip::{
    decode_gossip, delta_for, encode_gossip, validate_gossip_payload, GossipAction, SCORE_ACCEPT,
    SCORE_IGNORE, SCORE_REJECT,
};
pub use identity::NodeIdentity;
pub use multiaddr::{parse_quic_udp, QuicUdpAddr};
pub use peer_manager::{PeerManager, PeerRecord};
pub use reqresp::{handle_status, RequestTracker, StatusExchange};
pub use swarm::SwarmFacade;
pub use transport::{
    dial_quic, prepare_transport, reject_non_quic, BoundTransport, TransportConfig,
};
