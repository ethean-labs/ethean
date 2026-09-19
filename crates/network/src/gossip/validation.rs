//! Gossip validation outcomes mapped to ACCEPT / IGNORE / REJECT.

use ethean_network_wire::{decompress_raw, message_id, WireError};
use ethean_primitives::Hash32;

/// Gossipsub application validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GossipAction {
    /// Forward and deliver to the chain owner command path.
    Accept,
    /// Drop without penalizing (e.g. duplicate, unknown parent).
    Ignore,
    /// Drop and apply negative peer score.
    Reject,
}

/// Cheap gossip checks before consensus import.
pub fn validate_gossip_payload(
    topic: &str,
    compressed: &[u8],
    seen_ids: &mut std::collections::HashSet<Hash32>,
) -> (GossipAction, Option<Vec<u8>>) {
    let plain = match decompress_raw(compressed) {
        Ok(p) => p,
        Err(WireError::PayloadTooLarge { .. }) | Err(WireError::Snappy(_)) => {
            return (GossipAction::Reject, None);
        }
        Err(_) => return (GossipAction::Reject, None),
    };
    if topic.contains("/eth2/") || topic.is_empty() {
        return (GossipAction::Reject, None);
    }
    let id = message_id(topic, &plain);
    if !seen_ids.insert(id) {
        return (GossipAction::Ignore, None);
    }
    (GossipAction::Accept, Some(plain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::compress_raw;
    use std::collections::HashSet;

    #[test]
    fn accept_then_ignore_duplicate() {
        let topic = "/leanconsensus/abcd/block/ssz_snappy";
        let c = compress_raw(b"block-bytes").unwrap();
        let mut seen = HashSet::new();
        let (a1, p) = validate_gossip_payload(topic, &c, &mut seen);
        assert_eq!(a1, GossipAction::Accept);
        assert!(p.is_some());
        let (a2, _) = validate_gossip_payload(topic, &c, &mut seen);
        assert_eq!(a2, GossipAction::Ignore);
    }

    #[test]
    fn reject_eth2_topic() {
        let c = compress_raw(b"x").unwrap();
        let mut seen = HashSet::new();
        let (a, _) = validate_gossip_payload("/eth2/beacon_block/ssz_snappy", &c, &mut seen);
        assert_eq!(a, GossipAction::Reject);
    }
}
