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

/// Distinct attestation-data keys retained (same bound as the signature pool).
pub const MAX_POOL_KEYS: usize = super::MAX_TRACKED_DATA;

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
        if !self.entries.contains_key(&key) && self.entries.len() >= MAX_POOL_KEYS {
            if let Some(drop) = self.oldest_key() {
                self.entries.remove(&drop);
            }
        }
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

    fn oldest_key(&self) -> Option<PoolKey> {
        self.entries
            .iter()
            .min_by_key(|(_, list)| {
                list.iter()
                    .map(|e| e.inserted_slot)
                    .min()
                    .unwrap_or(u64::MAX)
            })
            .map(|(k, _)| *k)
    }

    /// Number of keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True when at least one verified Type-1 is pooled (a block body candidate).
    pub fn has_proved_entry(&self) -> bool {
        self.entries.values().flatten().any(|e| !e.proof.is_empty())
    }

    /// All retained variants for a key (insertion order). Callers that keep a
    /// proof clone only the entries they select.
    pub fn variants(&self, key: &PoolKey) -> Option<&[PoolEntry]> {
        self.entries.get(key).map(|v| v.as_slice())
    }

    /// Pool keys in message-root order (no proof copies).
    pub fn keys_by_message_root(&self) -> Vec<PoolKey> {
        let mut out: Vec<PoolKey> = self.entries.keys().copied().collect();
        out.sort_by_key(|k| k.message_root);
        out
    }

    /// Keys holding at least two variants, one of them inserted at or after
    /// `min_slot`, sorted by message root.
    pub fn mergeable_keys_since(&self, min_slot: u64) -> Vec<PoolKey> {
        let mut out: Vec<PoolKey> = self
            .entries
            .iter()
            .filter(|(_, list)| list.len() >= 2 && list.iter().any(|e| e.inserted_slot >= min_slot))
            .map(|(k, _)| *k)
            .collect();
        out.sort_by_key(|k| k.message_root);
        out
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
        out.sort_by_key(|a| a.0.message_root);
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
    fn bounds_distinct_keys() {
        let mut pool = AggregatePool::new(1);
        for i in 0..MAX_POOL_KEYS {
            let mut root = [0u8; 32];
            root[0] = (i / 256) as u8;
            root[1] = (i % 256) as u8;
            pool.insert_verified(
                PoolKey {
                    profile_digest: [1u8; 32],
                    message_root: root,
                },
                PoolEntry {
                    proof: vec![1],
                    coverage: 1,
                    inserted_slot: i as u64,
                    attestation_ssz: Vec::new(),
                },
            );
        }
        assert_eq!(pool.len(), MAX_POOL_KEYS);
        pool.insert_verified(
            PoolKey {
                profile_digest: [1u8; 32],
                message_root: [9u8; 32],
            },
            PoolEntry {
                proof: vec![2],
                coverage: 1,
                inserted_slot: MAX_POOL_KEYS as u64,
                attestation_ssz: Vec::new(),
            },
        );
        assert_eq!(pool.len(), MAX_POOL_KEYS);
        assert!(pool
            .best(&PoolKey {
                profile_digest: [1u8; 32],
                message_root: [9u8; 32],
            })
            .is_some());
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
        let keys = pool.keys_by_message_root();
        assert_eq!(keys[0].message_root, [3u8; 32]);
        assert_eq!(keys[1].message_root, [9u8; 32]);
    }

    #[test]
    fn mergeable_keys_need_two_variants_and_a_recent_one() {
        let mut pool = AggregatePool::new(4);
        let key = |root: u8| PoolKey {
            profile_digest: [1u8; 32],
            message_root: [root; 32],
        };
        let entry = |slot: u64| PoolEntry {
            proof: vec![slot as u8],
            coverage: 1,
            inserted_slot: slot,
            attestation_ssz: Vec::new(),
        };
        pool.insert_verified(key(5), entry(10));
        pool.insert_verified(key(5), entry(10));
        pool.insert_verified(key(3), entry(10));
        pool.insert_verified(key(7), entry(4));
        pool.insert_verified(key(7), entry(4));
        pool.insert_verified(key(1), entry(4));
        pool.insert_verified(key(1), entry(9));
        let keys: Vec<u8> = pool
            .mergeable_keys_since(8)
            .iter()
            .map(|k| k.message_root[0])
            .collect();
        assert_eq!(keys, vec![1, 5]);
    }
}
