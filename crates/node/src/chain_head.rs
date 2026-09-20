//! Canonical head updates and fork-choice view refresh on [`ChainOwner`].

use crate::chain_owner::ChainOwner;
use ethean_primitives::{Hash32, HASH32_ZERO};

impl ChainOwner {
    /// Move the canonical head; count a reorg when the new block does not extend it.
    pub fn advance_head(&mut self, new_root: Hash32, parent_root: Hash32) {
        let prev = self.head_root;
        if prev != HASH32_ZERO && prev != new_root && parent_root != prev {
            self.reorg_total = self.reorg_total.saturating_add(1);
        }
        self.head_root = new_root;
        self.refresh_fc_view();
    }

    /// Refresh justified / finalized / safe-target from the head post-state.
    ///
    /// Until a live [`ethean_fork_choice::ForkChoiceStore`] owns safe-target,
    /// safe-target tracks the justified checkpoint (conservative attest target).
    pub fn refresh_fc_view(&mut self) {
        let Some(state) = self.head_state.as_ref() else {
            self.safe_target = self.head_root;
            return;
        };
        self.safe_target = state.latest_justified.root;
    }

    /// Slot of the current safe-target root (justified slot while FC store is deferred).
    pub fn safe_target_slot(&self) -> u64 {
        self.head_state
            .as_ref()
            .map(|s| {
                if self.safe_target == s.latest_justified.root {
                    s.latest_justified.slot.get()
                } else if self.safe_target == self.head_root {
                    s.slot.get()
                } else {
                    s.latest_justified.slot.get()
                }
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_types::{
        BlockHeader, Checkpoint, GenesisConfig, State, Validator,
    };

    fn sample_state(slot: u64, justified: Hash32, just_slot: u64) -> State {
        let val = Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap();
        let mut st = State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::new(slot),
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint {
                root: justified,
                slot: Slot::new(just_slot),
            },
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![val],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        };
        let _ = &mut st;
        st
    }

    #[test]
    fn extension_is_not_a_reorg() {
        let mut owner = ChainOwner::new(2);
        let a = [1u8; 32];
        let b = [2u8; 32];
        owner.head_root = a;
        owner.advance_head(b, a);
        assert_eq!(owner.reorg_total, 0);
        assert_eq!(owner.head_root, b);
    }

    #[test]
    fn competing_parent_counts_reorg() {
        let mut owner = ChainOwner::new(2);
        let a = [1u8; 32];
        let side = [9u8; 32];
        let tip = [3u8; 32];
        owner.head_root = a;
        owner.advance_head(tip, side);
        assert_eq!(owner.reorg_total, 1);
    }

    #[test]
    fn safe_target_follows_justified() {
        let mut owner = ChainOwner::new(2);
        let just = [7u8; 32];
        owner.head_state = Some(sample_state(5, just, 4));
        owner.head_root = [5u8; 32];
        owner.refresh_fc_view();
        assert_eq!(owner.safe_target, just);
        assert_eq!(owner.safe_target_slot(), 4);
    }
}
