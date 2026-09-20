//! Block import into the fork-choice store.

use ethean_primitives::ValidatorIndex;
use ethean_types::{Block, State};

use crate::error::ForkChoiceError;
use crate::prune::prune_finalized_away;
use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Accept a block with a caller-supplied post-state (no STF re-run).
    pub fn on_block(&mut self, block: Block, post_state: State) -> Result<(), ForkChoiceError> {
        let block_root = block
            .hash_tree_root()
            .map_err(|e| ForkChoiceError::Types(e.to_string()))?;

        if self.blocks.contains_key(&block_root) {
            return Ok(());
        }

        let previous_finalized_slot = self.latest_finalized.slot;

        if !self.block_states.contains_key(&block.parent_root) {
            return Err(ForkChoiceError::UnknownParent);
        }

        let parent_state = &self.block_states[&block.parent_root];
        if block.slot.get().saturating_sub(parent_state.slot.get()) > self.historical_roots_limit
        {
            return Err(ForkChoiceError::BlockSlotGapTooLarge);
        }
        let current_slot = self.current_slot();
        if block.slot.get() > current_slot + 1 {
            return Err(ForkChoiceError::BlockTooFarInFuture);
        }

        let mut seen = std::collections::HashSet::new();
        for att in &block.body.attestations {
            let root = att.data.hash_tree_root();
            if !seen.insert(root) {
                return Err(ForkChoiceError::DuplicateAttestationData);
            }
        }

        // On-chain aggregates feed the known pool before head recompute (leanSpec / Gean).
        let body_votes: Vec<_> = block
            .body
            .attestations
            .iter()
            .map(|a| (a.aggregation_bits.bits.clone(), a.data))
            .collect();

        self.latest_justified = self.latest_justified.advance_to(post_state.latest_justified);
        self.blocks.insert(block_root, block);
        self.block_states.insert(block_root, post_state);

        for (bits, data) in body_votes {
            for (i, bit) in bits.iter().enumerate() {
                if *bit {
                    self.insert_known_vote(ValidatorIndex::new(i as u64), data);
                }
            }
            self.record_known_payload(data, &bits);
        }

        self.update_head()?;

        if self.latest_finalized.slot > previous_finalized_slot {
            prune_finalized_away(self);
            self.prune_stale_votes();
        }
        Ok(())
    }

    fn prune_stale_votes(&mut self) {
        let finalized = self.latest_finalized;
        let drop_new: Vec<_> = self
            .latest_new_attestations
            .iter()
            .filter(|(_, data)| {
                !(data.head.slot > finalized.slot
                    && self.checkpoint_is_ancestor(finalized, data.head))
            })
            .map(|(k, _)| *k)
            .collect();
        for k in drop_new {
            self.latest_new_attestations.remove(&k);
        }
        let drop_known: Vec<_> = self
            .latest_known_attestations
            .iter()
            .filter(|(_, data)| {
                !(data.head.slot > finalized.slot
                    && self.checkpoint_is_ancestor(finalized, data.head))
            })
            .map(|(k, _)| *k)
            .collect();
        for k in drop_known {
            self.latest_known_attestations.remove(&k);
        }
    }
}
