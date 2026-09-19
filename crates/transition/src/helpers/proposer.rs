//! Proposer schedule (round-robin).

use ethean_primitives::{Slot, ValidatorIndex};

use crate::error::TransitionError;

/// Validator assigned to propose at `slot` (`slot % num_validators`).
pub fn proposer_for_slot(
    slot: Slot,
    num_validators: u64,
) -> Result<ValidatorIndex, TransitionError> {
    if num_validators == 0 {
        return Err(TransitionError::EmptyValidatorRegistry(
            "Cannot schedule a proposer for an empty validator registry".into(),
        ));
    }
    Ok(ValidatorIndex::new(slot.get() % num_validators))
}

pub fn index_within_registry(index: ValidatorIndex, num_validators: u64) -> bool {
    index.get() < num_validators
}
