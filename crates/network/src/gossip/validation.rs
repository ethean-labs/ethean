//! Gossip validation outcomes mapped to ACCEPT / IGNORE / REJECT.

use std::collections::HashSet;

use ethean_network_wire::{decompress_raw, message_id_valid_snappy, WireError};
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

/// Message ids per generation of [`SeenIds`].
pub const SEEN_IDS_PER_GENERATION: usize = 1 << 16;

/// Delivered message ids, bounded to two generations: when the current one is
/// full it becomes the previous one and the oldest is dropped. A repeat older
/// than both generations is also caught by gossipsub's own duplicate cache.
#[derive(Debug)]
pub struct SeenIds {
    current: HashSet<Hash32>,
    previous: HashSet<Hash32>,
    per_generation: usize,
}

impl Default for SeenIds {
    fn default() -> Self {
        Self::with_generation_size(SEEN_IDS_PER_GENERATION)
    }
}

impl SeenIds {
    pub fn with_generation_size(per_generation: usize) -> Self {
        Self {
            current: HashSet::new(),
            previous: HashSet::new(),
            per_generation: per_generation.max(1),
        }
    }

    /// Record `id`; false when it was already seen.
    pub fn insert(&mut self, id: Hash32) -> bool {
        if self.previous.contains(&id) || !self.current.insert(id) {
            return false;
        }
        if self.current.len() >= self.per_generation {
            self.previous = std::mem::take(&mut self.current);
        }
        true
    }

    pub fn len(&self) -> usize {
        self.current.len() + self.previous.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Cheap gossip checks before consensus import.
pub fn validate_gossip_payload(
    topic: &str,
    compressed: &[u8],
    seen_ids: &mut SeenIds,
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
    let id20 = message_id_valid_snappy(topic, &plain);
    let mut id = [0u8; 32];
    id[..20].copy_from_slice(&id20);
    if !seen_ids.insert(id) {
        return (GossipAction::Ignore, None);
    }
    (GossipAction::Accept, Some(plain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_network_wire::compress_raw;

    #[test]
    fn accept_then_ignore_duplicate() {
        let topic = "/leanconsensus/abcd/block/ssz_snappy";
        let c = compress_raw(b"block-bytes").unwrap();
        let mut seen = SeenIds::default();
        let (a1, p) = validate_gossip_payload(topic, &c, &mut seen);
        assert_eq!(a1, GossipAction::Accept);
        assert!(p.is_some());
        let (a2, _) = validate_gossip_payload(topic, &c, &mut seen);
        assert_eq!(a2, GossipAction::Ignore);
    }

    #[test]
    fn reject_eth2_topic() {
        let c = compress_raw(b"x").unwrap();
        let mut seen = SeenIds::default();
        let (a, _) = validate_gossip_payload("/eth2/beacon_block/ssz_snappy", &c, &mut seen);
        assert_eq!(a, GossipAction::Reject);
    }

    #[test]
    fn seen_ids_keep_two_generations() {
        let mut seen = SeenIds::with_generation_size(2);
        let id = |b: u8| [b; 32];
        assert!(seen.insert(id(1)));
        assert!(seen.insert(id(2)));
        assert!(!seen.insert(id(1)), "previous generation still counts");
        assert!(seen.insert(id(3)));
        assert!(seen.insert(id(4)));
        assert!(seen.insert(id(1)), "dropped with the oldest generation");
        assert!(seen.len() <= 4);
    }
}
