//! Peer scoring and connection bookkeeping (disposable state).

use ethean_primitives::Hash32;
use std::collections::HashMap;

/// Per-peer score and last-seen slot claim (untrusted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerRecord {
    /// Opaque peer key (fingerprint).
    pub peer_id: Hash32,
    /// Gossip application score.
    pub score: i32,
    /// Last Status head slot claim.
    pub claimed_head_slot: u64,
}

/// In-memory peer table.
#[derive(Debug, Default)]
pub struct PeerManager {
    peers: HashMap<Hash32, PeerRecord>,
}

impl PeerManager {
    /// Insert or refresh a peer.
    pub fn upsert(&mut self, record: PeerRecord) {
        self.peers.insert(record.peer_id, record);
    }

    /// Apply application feedback after gossip validation.
    pub fn apply_feedback(&mut self, peer_id: &Hash32, delta: i32) {
        if let Some(p) = self.peers.get_mut(peer_id) {
            p.score = p.score.saturating_add(delta);
        }
    }

    /// Ensure the peer exists, then apply a score delta.
    pub fn ensure_and_feedback(&mut self, peer_id: Hash32, delta: i32) {
        self.peers.entry(peer_id).or_insert(PeerRecord {
            peer_id,
            score: 0,
            claimed_head_slot: 0,
        });
        self.apply_feedback(&peer_id, delta);
    }

    /// Read current score when the peer is known.
    pub fn score_of(&self, peer_id: &Hash32) -> Option<i32> {
        self.peers.get(peer_id).map(|p| p.score)
    }

    /// Remove peer on disconnect.
    pub fn remove(&mut self, peer_id: &Hash32) {
        self.peers.remove(peer_id);
    }

    /// Current peer count.
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_feedback() {
        let mut pm = PeerManager::default();
        let id = [7u8; 32];
        pm.upsert(PeerRecord {
            peer_id: id,
            score: 0,
            claimed_head_slot: 1,
        });
        pm.apply_feedback(&id, -10);
        assert_eq!(pm.peers.get(&id).unwrap().score, -10);
    }

    #[test]
    fn ensure_creates_then_scores() {
        let mut pm = PeerManager::default();
        let id = [3u8; 32];
        pm.ensure_and_feedback(id, SCORE_REJECT_PLACEHOLDER);
        assert_eq!(pm.score_of(&id), Some(-25));
    }
}

// Local alias so the test does not depend on gossip scoring imports.
const SCORE_REJECT_PLACEHOLDER: i32 = -25;
