//! LMD latest-vote extraction from aggregated payload pools (leanSpec).

use std::collections::HashMap;

use ethean_primitives::{Hash32, ValidatorIndex};
use ethean_types::AttestationData;

use crate::store::AggregatedPayloadEntry;
use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// leanSpec `_extract_attestations_from_aggregated_payloads` on the known pool.
    pub(crate) fn votes_from_known_payloads(
        &self,
    ) -> HashMap<ValidatorIndex, AttestationData> {
        extract_latest_votes(&self.latest_known_payloads, self.latest_finalized.slot.get())
    }

    /// Same extraction over the pending (new) payload pool — safe-target path.
    pub(crate) fn votes_from_new_payloads(
        &self,
    ) -> HashMap<ValidatorIndex, AttestationData> {
        extract_latest_votes(&self.latest_new_payloads, self.latest_finalized.slot.get())
    }
}

/// Newest-slot vote per validator; equal slot prefers larger attestation-data root.
fn extract_latest_votes(
    payloads: &HashMap<Hash32, AggregatedPayloadEntry>,
    finalized_slot: u64,
) -> HashMap<ValidatorIndex, AttestationData> {
    let mut ordered: Vec<&AggregatedPayloadEntry> = payloads.values().collect();
    ordered.sort_by(|a, b| {
        b.data
            .slot
            .cmp(&a.data.slot)
            .then_with(|| b.data_root.cmp(&a.data_root))
    });

    let mut latest: HashMap<ValidatorIndex, AttestationData> = HashMap::new();
    for entry in ordered {
        if entry.data.head.slot.get() <= finalized_slot {
            continue;
        }
        for set in &entry.participant_sets {
            for &idx in set {
                let v = ValidatorIndex::new(idx);
                latest.entry(v).or_insert(entry.data);
            }
        }
    }
    latest
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::HASH32_ZERO;
    use ethean_types::{Checkpoint, Slot};

    fn data(slot: u64, head_slot: u64, root_byte: u8) -> AttestationData {
        let mut root = HASH32_ZERO;
        root[0] = root_byte;
        AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint {
                root,
                slot: Slot::new(head_slot),
            },
            target: Checkpoint {
                root,
                slot: Slot::new(head_slot),
            },
            source: Checkpoint {
                root: HASH32_ZERO,
                slot: Slot::new(0),
            },
        }
    }

    #[test]
    fn newer_slot_wins_across_payloads() {
        let older = data(5, 4, 1);
        let newer = data(7, 6, 2);
        let mut map = HashMap::new();
        map.insert(
            older.hash_tree_root(),
            AggregatedPayloadEntry {
                data_root: older.hash_tree_root(),
                data: older,
                participant_sets: vec![vec![0, 1]],
            },
        );
        map.insert(
            newer.hash_tree_root(),
            AggregatedPayloadEntry {
                data_root: newer.hash_tree_root(),
                data: newer,
                participant_sets: vec![vec![0]],
            },
        );
        let votes = extract_latest_votes(&map, 3);
        assert_eq!(votes.get(&ValidatorIndex::new(0)).unwrap().slot.get(), 7);
        assert_eq!(votes.get(&ValidatorIndex::new(1)).unwrap().slot.get(), 5);
    }
}
