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

    /// Drop stale votes whose head is at or below the finalized slot.
    pub(crate) fn relevant_known_votes(
        &self,
    ) -> HashMap<ValidatorIndex, AttestationData> {
        self.latest_known_attestations
            .iter()
            .filter(|(_, data)| data.head.slot > self.latest_finalized.slot)
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    /// Relevant pending votes (for safe-target).
    pub(crate) fn relevant_new_votes(&self) -> HashMap<ValidatorIndex, AttestationData> {
        self.latest_new_attestations
            .iter()
            .filter(|(_, data)| data.head.slot > self.latest_finalized.slot)
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    /// Ancestor weights from the finalized checkpoint using known votes.
    ///
    /// Fixture `storeSnapshot.blockWeights` are keyed from the finalized floor
    /// (not the justified root), matching leanSpec store dumps.
    pub fn block_weights_from_known(&self) -> HashMap<Hash32, u64> {
        let start_slot = self
            .blocks
            .get(&self.latest_finalized.root)
            .map(|b| b.slot.get())
            .unwrap_or(0);
        self.accumulate_ancestor_weights(&self.relevant_known_votes(), start_slot)
    }
}
