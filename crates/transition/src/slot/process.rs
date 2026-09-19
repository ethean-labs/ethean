//! `process_slots` — advance empty slots up to (not including) the target.

use ethean_primitives::{Slot, HASH32_ZERO};
use ethean_types::State;

use crate::error::TransitionError;

/// Advance `state` through empty slots until `state.slot == target_slot`.
///
/// Mirrors leanSpec `process_slots`: target must be strictly in the future.
/// On the first empty slot after a block, cache `hash_tree_root(state)` into
/// `latest_block_header.state_root` when that field is still zero.
pub fn process_slots(state: &mut State, target_slot: Slot) -> Result<(), TransitionError> {
    if state.slot >= target_slot {
        return Err(TransitionError::BlockSlotNotInFuture(
            "Target slot must be in the future".into(),
        ));
    }

    while state.slot < target_slot {
        if state.latest_block_header.state_root == HASH32_ZERO {
            let root = state
                .hash_tree_root()
                .map_err(|e| TransitionError::Types(e.to_string()))?;
            state.latest_block_header.state_root = root;
        }
        state.slot = state
            .slot
            .checked_add(1)
            .map_err(|_| TransitionError::Types("slot overflow".into()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, ValidatorIndex};
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, Validator};

    fn sample_state(slot: u64, validators: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..validators {
            vals.push(
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64))
                    .unwrap(),
            );
        }
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::new(slot),
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vals,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn advances_n_slots_and_caches_state_root() {
        let mut state = sample_state(0, 1);
        assert_eq!(state.latest_block_header.state_root, HASH32_ZERO);
        process_slots(&mut state, Slot::new(3)).unwrap();
        assert_eq!(state.slot, Slot::new(3));
        assert_ne!(state.latest_block_header.state_root, HASH32_ZERO);
    }

    #[test]
    fn rejects_non_future_target() {
        let mut state = sample_state(5, 1);
        let err = process_slots(&mut state, Slot::new(5)).unwrap_err();
        assert!(matches!(err, TransitionError::BlockSlotNotInFuture(_)));
        let err = process_slots(&mut state, Slot::new(4)).unwrap_err();
        assert!(matches!(err, TransitionError::BlockSlotNotInFuture(_)));
    }
}
