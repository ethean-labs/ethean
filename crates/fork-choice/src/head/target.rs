//! Attestation target and source (leanSpec lstar `validator_duties.py`).

use ethean_primitives::{Hash32, HASH32_ZERO};
use ethean_transition::is_justifiable_after;
use ethean_types::Checkpoint;

use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// leanSpec `get_attestation_target`.
    ///
    /// Walks back from the head at most `lookback` steps while above
    /// `max(safe_target slot, finalized slot)`, then on until the slot is
    /// justifiable after the finalized slot. Never goes past finalization.
    pub fn attestation_target(&self, lookback: u64) -> Checkpoint {
        let finalized = self.latest_finalized.slot;
        let safe_slot = self
            .blocks
            .get(&self.safe_target)
            .map(|b| b.slot)
            .unwrap_or(finalized);
        let lower_bound = safe_slot.max(finalized);

        let mut root = self.head;
        for _ in 0..lookback {
            match self.blocks.get(&root) {
                Some(block) if block.slot > lower_bound => root = block.parent_root,
                _ => break,
            }
        }
        while let Some(block) = self.blocks.get(&root) {
            if block.slot <= finalized || is_justifiable_after(block.slot, finalized) {
                return Checkpoint {
                    root,
                    slot: block.slot,
                };
            }
            if !self.blocks.contains_key(&block.parent_root) {
                break;
            }
            root = block.parent_root;
        }
        self.latest_finalized
    }

    /// leanSpec `produce_attestation_data` source: the head state's justified
    /// checkpoint, with the genesis placeholder root replaced by the head.
    pub fn attestation_source(&self) -> Option<Checkpoint> {
        let justified = self.block_states.get(&self.head)?.latest_justified;
        if justified.root == HASH32_ZERO {
            return Some(Checkpoint {
                root: self.head,
                slot: justified.slot,
            });
        }
        Some(justified)
    }

    /// Slot of a block held by the store.
    pub fn block_slot(&self, root: &Hash32) -> Option<u64> {
        self.blocks.get(root).map(|b| b.slot.get())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::chain_store;
    use ethean_primitives::Slot;

    #[test]
    fn target_walks_back_to_a_justifiable_slot() {
        // Blocks at slots 0..=8 in a line; head at 8, safe target at 8.
        let (mut store, roots) = chain_store(8);
        store.safe_target = roots[8];
        // Slot 8 is not justifiable after 0 (7 and 8 are neither <=5, squares nor pronic).
        let target = store.attestation_target(3);
        assert_eq!(target.slot, Slot::new(6));
        assert_eq!(target.root, roots[6]);
    }

    #[test]
    fn target_stays_within_the_lookback_toward_the_safe_target() {
        let (mut store, roots) = chain_store(8);
        store.safe_target = roots[0];
        // Three steps back from 8 reach 5, which is justifiable after 0.
        let target = store.attestation_target(3);
        assert_eq!(target.slot, Slot::new(5));
        assert_eq!(target.root, roots[5]);
    }

    #[test]
    fn source_replaces_the_genesis_placeholder_root() {
        let (store, roots) = chain_store(0);
        let source = store.attestation_source().unwrap();
        assert_eq!(source.root, roots[0]);
        assert_eq!(source.slot, Slot::ZERO);
    }
}
