//! Ancestor weight accumulation for the head walk.

use std::collections::HashMap;

use ethean_primitives::{Hash32, ValidatorIndex};
use ethean_types::AttestationData;

use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Credit each block above `start_slot` for every latest vote that lands on it.
    pub(crate) fn accumulate_ancestor_weights(
        &self,
        attestations: &HashMap<ValidatorIndex, AttestationData>,
        start_slot: u64,
    ) -> HashMap<Hash32, u64> {
        let mut weights: HashMap<Hash32, u64> = HashMap::new();
        for data in attestations.values() {
            let mut current_root = data.head.root;
            while let Some(block) = self.blocks.get(&current_root) {
                if block.slot.get() <= start_slot {
                    break;
                }
                *weights.entry(current_root).or_insert(0) += 1;
                current_root = block.parent_root;
            }
        }
        weights
    }

    /// Counted votes: prefer payload-pool LMD (leanSpec); fall back to the map.
    pub(crate) fn relevant_known_votes(
        &self,
    ) -> HashMap<ValidatorIndex, AttestationData> {
        let from_payloads = self.votes_from_known_payloads();
        if !from_payloads.is_empty() {
            return from_payloads;
        }
        self.latest_known_attestations
            .iter()
            .filter(|(_, data)| data.head.slot > self.latest_finalized.slot)
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    /// Pending votes for safe-target (payload pool first).
    pub(crate) fn relevant_new_votes(&self) -> HashMap<ValidatorIndex, AttestationData> {
        let from_payloads = self.votes_from_new_payloads();
        if !from_payloads.is_empty() {
            return from_payloads;
        }
        self.latest_new_attestations
            .iter()
            .filter(|(_, data)| data.head.slot > self.latest_finalized.slot)
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    /// Ancestor weights from the finalized checkpoint (leanSpec `compute_block_weights`).
    pub fn block_weights_from_known(&self) -> HashMap<Hash32, u64> {
        let start_slot = self.latest_finalized.slot.get();
        self.accumulate_ancestor_weights(&self.relevant_known_votes(), start_slot)
    }
}
