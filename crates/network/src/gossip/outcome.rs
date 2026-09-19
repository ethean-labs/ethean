//! Structured swarm pump outcomes (gossip validation + opaque kinds).

use crate::gossip::validation::GossipAction;
use ethean_primitives::Hash32;

/// One drained swarm event after optional gossip validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PumpEvent {
    /// QUIC connection opened (peer fingerprint when known).
    ConnectionEstablished {
        /// SHA-256 of PeerId bytes when available.
        peer: Option<Hash32>,
    },
    /// QUIC connection closed (peer fingerprint when known).
    ConnectionClosed {
        /// SHA-256 of PeerId bytes when available.
        peer: Option<Hash32>,
    },
    /// Outbound dial failed.
    OutgoingError,
    /// Inbound connection failed.
    IncomingError,
    /// New listen multiaddr.
    NewListenAddr,
    /// Validated gossipsub message (may carry plain SSZ after Snappy).
    Gossip(GossipIngress),
    /// Gossipsub subscribe ack.
    GossipSubscribed,
    /// Gossipsub unsubscribe ack.
    GossipUnsubscribed,
    /// Behaviour event other than classified gossip.
    Behaviour,
    /// Anything else from the swarm.
    Other,
}

/// Inbound gossip after Lean validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipIngress {
    /// ACCEPT / IGNORE / REJECT.
    pub action: GossipAction,
    /// Gossip topic string.
    pub topic: String,
    /// Optional source peer fingerprint (SHA-256 of PeerId bytes).
    pub peer: Option<Hash32>,
    /// Decompressed SSZ when action is ACCEPT.
    pub plain: Option<Vec<u8>>,
}

impl PumpEvent {
    /// Stable short label for logs and legacy callers.
    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::ConnectionEstablished { .. } => "connection_established",
            Self::ConnectionClosed { .. } => "connection_closed",
            Self::OutgoingError => "outgoing_error",
            Self::IncomingError => "incoming_error",
            Self::NewListenAddr => "new_listen_addr",
            Self::Gossip(g) => match g.action {
                GossipAction::Accept => "gossip_accept",
                GossipAction::Ignore => "gossip_ignore",
                GossipAction::Reject => "gossip_reject",
            },
            Self::GossipSubscribed => "gossip_subscribed",
            Self::GossipUnsubscribed => "gossip_unsubscribed",
            Self::Behaviour => "behaviour",
            Self::Other => "other",
        }
    }

    /// Gossip ingress when this event carries one.
    pub fn gossip(&self) -> Option<&GossipIngress> {
        match self {
            Self::Gossip(g) => Some(g),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_label() {
        let ev = PumpEvent::Gossip(GossipIngress {
            action: GossipAction::Accept,
            topic: "/leanconsensus/x/block/ssz_snappy".into(),
            peer: None,
            plain: Some(vec![1, 2, 3]),
        });
        assert_eq!(ev.kind_label(), "gossip_accept");
        assert!(ev.gossip().unwrap().plain.is_some());
    }
}
