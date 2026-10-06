//! Pending req/resp request tracker with disconnect pruning.

use ethean_primitives::Hash32;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Requests without a response after this long are dropped from the tracker.
/// The swarm reports no event for a failed outbound request, so without it
/// such entries would stay until the peer disconnects.
pub const REQUEST_EXPIRY: Duration = Duration::from_secs(30);

/// One in-flight request id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Tracks pending requests per peer until response, disconnect or expiry.
#[derive(Debug, Default)]
pub struct RequestTracker {
    next_id: u64,
    pending: HashMap<RequestId, (Hash32, Instant)>,
}

impl RequestTracker {
    /// Allocate a request id bound to `peer`.
    pub fn insert(&mut self, peer: Hash32) -> RequestId {
        let id = RequestId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.pending.insert(id, (peer, Instant::now()));
        id
    }

    /// Complete a request.
    pub fn complete(&mut self, id: RequestId) -> Option<Hash32> {
        self.pending.remove(&id).map(|(peer, _)| peer)
    }

    /// Complete the oldest open request to `peer` (responses carry the peer,
    /// not our id; each request gets one response).
    pub fn complete_oldest(&mut self, peer: &Hash32) -> Option<RequestId> {
        let id = self
            .pending
            .iter()
            .filter(|(_, (p, _))| p == peer)
            .map(|(id, _)| *id)
            .min_by_key(|id| id.0)?;
        self.pending.remove(&id);
        Some(id)
    }

    /// Drop all requests for a disconnected peer.
    pub fn prune_peer(&mut self, peer: &Hash32) {
        self.pending.retain(|_, (p, _)| p != peer);
    }

    /// Drop requests older than `max_age`; returns how many were dropped.
    pub fn expire(&mut self, now: Instant, max_age: Duration) -> usize {
        let before = self.pending.len();
        self.pending
            .retain(|_, (_, at)| now.saturating_duration_since(*at) < max_age);
        before - self.pending.len()
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

    #[test]
    fn response_closes_the_oldest_request_of_that_peer() {
        let mut t = RequestTracker::default();
        let (a, b) = ([1u8; 32], [2u8; 32]);
        let first = t.insert(a);
        let _other = t.insert(b);
        let second = t.insert(a);
        assert_eq!(t.complete_oldest(&a), Some(first));
        assert_eq!(t.complete_oldest(&a), Some(second));
        assert_eq!(t.complete_oldest(&a), None);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn unanswered_requests_expire() {
        let mut t = RequestTracker::default();
        let _ = t.insert([3u8; 32]);
        assert_eq!(t.expire(Instant::now(), REQUEST_EXPIRY), 0);
        assert_eq!(t.expire(Instant::now() + REQUEST_EXPIRY, REQUEST_EXPIRY), 1);
        assert!(t.is_empty());
    }
}
