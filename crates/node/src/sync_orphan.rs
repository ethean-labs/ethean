//! Pending sync blocks waiting for a missing parent (multi-hop catch-up).

use ethean_primitives::Hash32;
use std::collections::HashMap;

/// Max orphan SignedBlock blobs retained for parent catch-up.
pub const MAX_SYNC_ORPHANS: usize = 64;

/// One decoded sync blob that could not import yet (parent ≠ head).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOrphan {
    /// Missing parent root required before import.
    pub parent: Hash32,
    /// Raw SSZ (`SignedBlock` or `Block`).
    pub blob: Vec<u8>,
}

/// In-memory orphan map keyed by block root.
#[derive(Debug, Default, Clone)]
pub struct SyncOrphanCache {
    by_root: HashMap<Hash32, SyncOrphan>,
}

impl SyncOrphanCache {
    /// Number of buffered orphans.
    pub fn len(&self) -> usize {
        self.by_root.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.by_root.is_empty()
    }

    /// Insert or replace; drops oldest-ish entry when over cap (arbitrary HashMap eviction).
    pub fn insert(&mut self, root: Hash32, orphan: SyncOrphan) {
        if self.by_root.len() >= MAX_SYNC_ORPHANS && !self.by_root.contains_key(&root) {
            if let Some(evict) = self.by_root.keys().next().copied() {
                self.by_root.remove(&evict);
            }
        }
        self.by_root.insert(root, orphan);
    }

    /// Remove and return one orphan whose parent equals `head`.
    pub fn take_child_of(&mut self, head: Hash32) -> Option<(Hash32, SyncOrphan)> {
        let root = self
            .by_root
            .iter()
            .find(|(_, o)| o.parent == head)
            .map(|(r, _)| *r)?;
        self.by_root.remove(&root).map(|o| (root, o))
    }

    /// Unique missing parents that are not the zero hash.
    pub fn missing_parents(&self) -> Vec<Hash32> {
        let mut out = Vec::new();
        for o in self.by_root.values() {
            if o.parent == Hash32::default() {
                continue;
            }
            if !out.contains(&o.parent) {
                out.push(o.parent);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_child_matches_parent() {
        let mut c = SyncOrphanCache::default();
        c.insert(
            [2u8; 32],
            SyncOrphan {
                parent: [1u8; 32],
                blob: vec![9],
            },
        );
        assert!(c.take_child_of([0u8; 32]).is_none());
        let (root, o) = c.take_child_of([1u8; 32]).unwrap();
        assert_eq!(root, [2u8; 32]);
        assert_eq!(o.blob, vec![9]);
        assert!(c.is_empty());
    }
}
