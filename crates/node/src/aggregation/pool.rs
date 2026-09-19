//! Bounded aggregate proof pool keyed by profile + attestation-data root.

use ethean_primitives::Hash32;
use std::collections::HashMap;

/// Pool key: profile digest + message / attestation-data root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolKey {
    /// Profile / aggregation fingerprint digest.
    pub profile_digest: Hash32,
    /// Canonical attestation-data or block binding root.
    pub message_root: Hash32,
}

/// One retained proof variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolEntry {
    /// Proof bytes (already locally verified before insert).
    pub proof: Vec<u8>,
    /// Coverage participant count (informational).
    pub coverage: u32,
    /// Insertion slot for TTL / staleness.
    pub inserted_slot: u64,
}

/// In-memory pool with per-key variant bounds.
#[derive(Debug, Default)]
pub struct AggregatePool {
    entries: HashMap<PoolKey, Vec<PoolEntry>>,
    max_variants: usize,
}

impl AggregatePool {
    /// Create a pool with a per-key variant cap.
    pub fn new(max_variants_per_key: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_variants: max_variants_per_key.max(1),
        }
    }

    /// Insert only after local verify; drops oldest when over cap.
    pub fn insert_verified(&mut self, key: PoolKey, entry: PoolEntry) {
        let list = self.entries.entry(key).or_default();
        list.push(entry);
        while list.len() > self.max_variants {
            list.remove(0);
        }
    }

    /// Best coverage entry for a key, if any.
    pub fn best(&self, key: &PoolKey) -> Option<&PoolEntry> {
        self.entries
            .get(key)
            .and_then(|v| v.iter().max_by_key(|e| e.coverage))
    }

    /// Drop entries older than `min_slot`.
    pub fn prune_before(&mut self, min_slot: u64) {
        for list in self.entries.values_mut() {
            list.retain(|e| e.inserted_slot >= min_slot);
        }
        self.entries.retain(|_, v| !v.is_empty());
    }

    /// Number of keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_variant_cap() {
        let mut pool = AggregatePool::new(2);
        let key = PoolKey {
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
        };
        for i in 0..3 {
            pool.insert_verified(
                key,
                PoolEntry {
                    proof: vec![i],
                    coverage: i as u32,
                    inserted_slot: i as u64,
                },
            );
        }
        assert_eq!(pool.best(&key).unwrap().coverage, 2);
        assert_eq!(pool.entries.get(&key).unwrap().len(), 2);
    }
}
