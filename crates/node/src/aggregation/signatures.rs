//! Verified individual attestation signatures awaiting aggregation
//! (leanSpec `Store.attestation_signatures`).
//!
//! Only aggregators keep signatures. Entries are grouped by attestation data
//! root and bounded per root (one per validator) and in total.

use std::collections::{BTreeMap, HashMap};

use ethean_crypto::Signature;
use ethean_primitives::Hash32;
use ethean_types::AttestationData;

/// Upper bound on distinct attestation data retained at once.
pub const MAX_TRACKED_DATA: usize = 256;

#[derive(Debug, Clone)]
struct Votes {
    data: AttestationData,
    /// Validator index -> signature, ordered for deterministic aggregation.
    signatures: BTreeMap<u64, Signature>,
}

/// Signatures grouped by the attestation data they sign.
#[derive(Debug, Default)]
pub struct AttestationSignaturePool {
    by_root: HashMap<Hash32, Votes>,
}

impl AttestationSignaturePool {
    /// Record a verified signature. Returns false when the pool is full for a
    /// new data root or the validator already has a signature for this data.
    pub fn insert(
        &mut self,
        data_root: Hash32,
        data: &AttestationData,
        validator_index: u64,
        signature: Signature,
    ) -> bool {
        if !self.by_root.contains_key(&data_root) && self.by_root.len() >= MAX_TRACKED_DATA {
            return false;
        }
        let votes = self.by_root.entry(data_root).or_insert_with(|| Votes {
            data: *data,
            signatures: BTreeMap::new(),
        });
        if votes.signatures.contains_key(&validator_index) {
            return false;
        }
        votes.signatures.insert(validator_index, signature);
        true
    }

    /// Attestation data for a root, if tracked.
    pub fn data(&self, data_root: &Hash32) -> Option<&AttestationData> {
        self.by_root.get(data_root).map(|v| &v.data)
    }

    /// Signatures for `data_root` from validators not in `covered`, by index.
    pub fn uncovered(&self, data_root: &Hash32, covered: &[bool]) -> Vec<(u64, Signature)> {
        let Some(votes) = self.by_root.get(data_root) else {
            return Vec::new();
        };
        votes
            .signatures
            .iter()
            .filter(|(index, _)| !covered.get(**index as usize).copied().unwrap_or(false))
            .map(|(index, sig)| (*index, sig.clone()))
            .collect()
    }

    /// Roots with at least one retained signature, sorted for determinism.
    pub fn roots(&self) -> Vec<Hash32> {
        let mut roots: Vec<Hash32> = self.by_root.keys().copied().collect();
        roots.sort();
        roots
    }

    /// Drop signatures absorbed by a proof covering `participants`; votes that
    /// arrived while the proof was being built stay for the next round.
    pub fn remove_covered(&mut self, data_root: &Hash32, participants: &[bool]) {
        let Some(votes) = self.by_root.get_mut(data_root) else {
            return;
        };
        votes
            .signatures
            .retain(|index, _| !participants.get(*index as usize).copied().unwrap_or(false));
        if votes.signatures.is_empty() {
            self.by_root.remove(data_root);
        }
    }

    /// Drop votes for slots before `min_slot`.
    pub fn prune_before(&mut self, min_slot: u64) {
        self.by_root.retain(|_, v| v.data.slot.get() >= min_slot);
    }

    pub fn len(&self) -> usize {
        self.by_root.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_root.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::SIGNATURE_BYTES;
    use ethean_primitives::Slot;
    use ethean_types::Checkpoint;

    fn data(slot: u64) -> AttestationData {
        AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        }
    }

    fn sig(b: u8) -> Signature {
        Signature::from_bytes([b; SIGNATURE_BYTES])
    }

    #[test]
    fn keeps_one_signature_per_validator_and_filters_covered() {
        let mut pool = AttestationSignaturePool::default();
        let d = data(3);
        let root = d.hash_tree_root();
        assert!(pool.insert(root, &d, 2, sig(2)));
        assert!(pool.insert(root, &d, 0, sig(0)));
        assert!(!pool.insert(root, &d, 2, sig(9)), "duplicate vote rejected");
        let all: Vec<u64> = pool.uncovered(&root, &[]).iter().map(|(i, _)| *i).collect();
        assert_eq!(all, vec![0, 2]);
        let rest: Vec<u64> = pool
            .uncovered(&root, &[true])
            .iter()
            .map(|(i, _)| *i)
            .collect();
        assert_eq!(rest, vec![2]);
        pool.prune_before(4);
        assert!(pool.is_empty());
    }

    #[test]
    fn bounds_distinct_data() {
        let mut pool = AttestationSignaturePool::default();
        for slot in 0..MAX_TRACKED_DATA as u64 {
            let d = data(slot);
            assert!(pool.insert(d.hash_tree_root(), &d, 0, sig(1)));
        }
        let extra = data(MAX_TRACKED_DATA as u64);
        assert!(!pool.insert(extra.hash_tree_root(), &extra, 0, sig(1)));
    }
}
