//! Block header validation and header-linked state updates.

use ethean_primitives::{Slot, HASH32_ZERO};
use ethean_types::{Block, BlockHeader, Checkpoint, State};

use crate::error::TransitionError;
use crate::helpers::{extend_to_slot, proposer_for_slot};

/// Validate the block header and update header-linked state (leanSpec `process_block_header`).
pub fn process_block_header(state: &mut State, block: &Block) -> Result<(), TransitionError> {
    let parent_header = state.latest_block_header.clone();
    let parent_root = parent_header.hash_tree_root();

    if block.slot != state.slot {
        return Err(TransitionError::BlockSlotMismatch(
            "Block slot mismatch".into(),
        ));
    }

    if block.slot.get() <= parent_header.slot.get() {
        return Err(TransitionError::BlockOlderThanLatestHeader(
            "Block is older than latest header".into(),
        ));
    }

    let num_validators = state.validators.len() as u64;
    let expected = proposer_for_slot(state.slot, num_validators)?;
    if block.proposer_index != expected {
        return Err(TransitionError::WrongProposer(
            "Incorrect block proposer".into(),
        ));
    }

    if block.proposer_index.get() >= num_validators {
        return Err(TransitionError::ProposerIndexOutOfRange(
            "Proposer index out of range".into(),
        ));
    }

    if block.parent_root != parent_root {
        return Err(TransitionError::InvalidParent(
            "Block parent root mismatch".into(),
        ));
    }

    if parent_header.slot == Slot::ZERO {
        state.latest_justified = Checkpoint {
            slot: Slot::ZERO,
            root: parent_root,
        };
        state.latest_finalized = Checkpoint {
            slot: Slot::ZERO,
            root: parent_root,
        };
    }

    let num_empty_slots = block
        .slot
        .get()
        .saturating_sub(parent_header.slot.get())
        .saturating_sub(1) as usize;

    state.historical_block_hashes.push(parent_root);
    for _ in 0..num_empty_slots {
        state.historical_block_hashes.push(HASH32_ZERO);
    }

    let last_materialized = block
        .slot
        .checked_sub(1)
        .map_err(|_| TransitionError::Types("block slot underflow when materializing".into()))?;
    state.justified_slots = extend_to_slot(
        &state.justified_slots,
        state.latest_finalized.slot,
        last_materialized,
    );

    let body_root = block
        .body
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;

    state.latest_block_header = BlockHeader {
        slot: block.slot,
        proposer_index: block.proposer_index,
        parent_root: block.parent_root,
        body_root,
        state_root: HASH32_ZERO,
    };

    Ok(())
}
