//! Pending req/resp request tracker with disconnect pruning.

use ethean_primitives::Hash32;
use std::collections::HashMap;

/// One in-flight request id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Tracks pending requests per peer until response or disconnect.
#[derive(Debug, Default)]
pub struct RequestTracker {
    next_id: u64,
    pending: HashMap<RequestId, Hash32>,
}

impl RequestTracker {
    /// Allocate a request id bound to `peer`.
    pub fn insert(&mut self, peer: Hash32) -> RequestId {
        let id = RequestId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.pending.insert(id, peer);
        id
    }

    /// Complete a request.
    pub fn complete(&mut self, id: RequestId) -> Option<Hash32> {
        self.pending.remove(&id)
    }

    /// Drop all requests for a disconnected peer.
    pub fn prune_peer(&mut self, peer: &Hash32) {
        self.pending.retain(|_, p| p != peer);
    }

    /// Pending count.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prune_on_disconnect() {
        let mut t = RequestTracker::default();
        let peer = [9u8; 32];
        let _ = t.insert(peer);
        t.prune_peer(&peer);
        assert!(t.is_empty());
    }
}
