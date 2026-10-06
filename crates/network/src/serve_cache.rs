//! Blocks served on inbound blocks-by-root / blocks-by-range requests.
//!
//! Bytes are kept in memory only for the most recent slots; older blocks are
//! read through an optional loader (the node's `blocks/<root>.ssz` files). The
//! slot → root index is small and covers leanSpec's serve window.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;

use ethean_primitives::Hash32;

/// Slots whose block bytes stay in memory (a block with its proof is ~240 KB).
pub const SERVE_BYTES_SLOTS: u64 = 64;

/// Slots indexed by root for range requests (leanSpec
/// `MIN_SLOTS_FOR_BLOCK_REQUESTS`).
pub const SERVE_INDEX_SLOTS: u64 = 3600;

/// Blocks put without a slot (by-root only) kept in memory.
const UNSLOTTED_CAP: usize = 64;

/// Reads a stored block by root when it is no longer in memory.
pub type BlockLoader = Arc<dyn Fn(&Hash32) -> Option<Vec<u8>> + Send + Sync>;

#[derive(Default)]
pub struct ServeCache {
    bytes: HashMap<Hash32, Vec<u8>>,
    by_slot: BTreeMap<u64, Hash32>,
    unslotted: VecDeque<Hash32>,
    /// Slots below this no longer keep bytes in memory.
    bytes_floor: u64,
    loader: Option<BlockLoader>,
}

impl std::fmt::Debug for ServeCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServeCache")
            .field("bytes", &self.bytes.len())
            .field("slots", &self.by_slot.len())
            .field("loader", &self.loader.is_some())
            .finish()
    }
}

impl ServeCache {
    pub fn set_loader(&mut self, loader: BlockLoader) {
        self.loader = Some(loader);
    }

    /// Block known by root only (no slot).
    pub fn put(&mut self, root: Hash32, bytes: Vec<u8>) {
        if self.bytes.insert(root, bytes).is_none() {
            self.unslotted.push_back(root);
        }
        while self.unslotted.len() > UNSLOTTED_CAP {
            if let Some(old) = self.unslotted.pop_front() {
                if !self.slot_bytes_kept(&old) {
                    self.bytes.remove(&old);
                }
            }
        }
    }

    /// Block at `slot`; replaces an earlier block indexed at the same slot.
    pub fn put_at_slot(&mut self, slot: u64, root: Hash32, bytes: Vec<u8>) {
        if let Some(prev) = self.by_slot.insert(slot, root) {
            if prev != root && !self.unslotted.contains(&prev) {
                self.bytes.remove(&prev);
            }
        }
        self.advance_floors();
        if slot >= self.bytes_floor {
            self.bytes.insert(root, bytes);
        }
    }

    pub fn get(&self, root: &Hash32) -> Option<Vec<u8>> {
        if let Some(b) = self.bytes.get(root) {
            return Some(b.clone());
        }
        self.loader.as_ref().and_then(|load| load(root))
    }

    pub fn get_at_slot(&self, slot: u64) -> Option<Vec<u8>> {
        self.by_slot.get(&slot).and_then(|root| self.get(root))
    }

    /// Indexed slots.
    pub fn slots(&self) -> usize {
        self.by_slot.len()
    }

    /// Blocks held in memory.
    pub fn resident(&self) -> usize {
        self.bytes.len()
    }

    fn newest_slot(&self) -> u64 {
        self.by_slot.keys().next_back().copied().unwrap_or(0)
    }

    fn slot_bytes_kept(&self, root: &Hash32) -> bool {
        self.by_slot
            .range(self.bytes_floor..)
            .any(|(_, r)| r == root)
    }

    fn advance_floors(&mut self) {
        let newest = self.newest_slot();
        let index_floor = newest.saturating_sub(SERVE_INDEX_SLOTS);
        while self
            .by_slot
            .first_key_value()
            .is_some_and(|(s, _)| *s < index_floor)
        {
            if let Some((_, root)) = self.by_slot.pop_first() {
                if !self.unslotted.contains(&root) {
                    self.bytes.remove(&root);
                }
            }
        }
        let bytes_floor = newest.saturating_sub(SERVE_BYTES_SLOTS);
        if bytes_floor <= self.bytes_floor {
            return;
        }
        let stale: Vec<Hash32> = self
            .by_slot
            .range(self.bytes_floor..bytes_floor)
            .map(|(_, r)| *r)
            .filter(|r| !self.unslotted.contains(r))
            .collect();
        for root in stale {
            self.bytes.remove(&root);
        }
        self.bytes_floor = bytes_floor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(slot: u64) -> Hash32 {
        let mut r = [0u8; 32];
        r[..8].copy_from_slice(&slot.to_le_bytes());
        r
    }

    #[test]
    fn old_blocks_leave_memory_but_stay_indexed() {
        let mut cache = ServeCache::default();
        for slot in 0..=200 {
            cache.put_at_slot(slot, root(slot), vec![slot as u8]);
        }
        assert_eq!(cache.slots(), 201);
        assert_eq!(cache.resident(), SERVE_BYTES_SLOTS as usize + 1);
        assert_eq!(cache.get_at_slot(200), Some(vec![200]));
        assert_eq!(cache.get_at_slot(10), None, "no loader");

        cache.set_loader(Arc::new(|r: &Hash32| Some(vec![r[0], 0xee])));
        assert_eq!(cache.get_at_slot(10), Some(vec![10, 0xee]));
        assert_eq!(cache.get(&root(199)), Some(vec![199]), "memory first");
    }

    #[test]
    fn index_is_bounded_to_the_serve_window() {
        let mut cache = ServeCache::default();
        cache.put_at_slot(1, root(1), vec![1]);
        cache.put_at_slot(SERVE_INDEX_SLOTS + 10, root(2), vec![2]);
        assert_eq!(cache.slots(), 1);
        assert_eq!(cache.resident(), 1);
    }

    #[test]
    fn unslotted_blocks_are_capped() {
        let mut cache = ServeCache::default();
        for i in 0..(UNSLOTTED_CAP as u64 + 10) {
            cache.put(root(1000 + i), vec![1]);
        }
        assert_eq!(cache.resident(), UNSLOTTED_CAP);
        assert_eq!(cache.get(&root(1000)), None);
    }
}
