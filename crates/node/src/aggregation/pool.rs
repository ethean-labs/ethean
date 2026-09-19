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
    /// SSZ of `AggregatedAttestation` when gossip carried attestation data.
    pub attestation_ssz: Vec<u8>,
}

/// In-memory pool with per-key variant bounds.
#[derive(Debug)]
pub struct AggregatePool {
    entries: HashMap<PoolKey, Vec<PoolEntry>>,
    max_variants: usize,
}

impl Default for AggregatePool {
    fn default() -> Self {
        Self::new(8)
    }
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

    /// Best-coverage entry per key, sorted by message root for deterministic builds.
    pub fn best_entries(&self) -> Vec<(PoolKey, PoolEntry)> {
        let mut out: Vec<(PoolKey, PoolEntry)> = self
            .entries
            .iter()
            .filter_map(|(k, list)| {
                list.iter()
                    .max_by_key(|e| e.coverage)
                    .map(|e| (*k, e.clone()))
            })
            .collect();
        out.sort_by(|a, b| a.0.message_root.cmp(&b.0.message_root));
        out
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
                    attestation_ssz: Vec::new(),
                },
            );
        }
        assert_eq!(pool.best(&key).unwrap().coverage, 2);
        assert_eq!(pool.entries.get(&key).unwrap().len(), 2);
    }

    #[test]
    fn best_entries_sort_by_message_root() {
        let mut pool = AggregatePool::new(2);
        let k1 = PoolKey {
            profile_digest: [1u8; 32],
            message_root: [9u8; 32],
        };
        let k2 = PoolKey {
            profile_digest: [1u8; 32],
            message_root: [3u8; 32],
        };
        pool.insert_verified(
            k1,
            PoolEntry {
                proof: vec![1],
                coverage: 1,
                inserted_slot: 1,
                attestation_ssz: Vec::new(),
            },
        );
        pool.insert_verified(
            k2,
            PoolEntry {
                proof: vec![2],
                coverage: 4,
                inserted_slot: 2,
                attestation_ssz: Vec::new(),
            },
        );
        let best = pool.best_entries();
        assert_eq!(best.len(), 2);
        assert_eq!(best[0].0.message_root, [3u8; 32]);
        assert_eq!(best[1].0.message_root, [9u8; 32]);
    }
}
