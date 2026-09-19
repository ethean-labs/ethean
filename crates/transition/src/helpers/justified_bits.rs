//! Justified-slot bitlist helpers (leanSpec `JustifiedSlots`).

use ethean_primitives::Slot;

use crate::error::TransitionError;
use crate::helpers::slot_math::justified_index_after;

/// Whether `target` is justified relative to `finalized` and the bitlist.
pub fn is_slot_justified(
    bits: &[bool],
    finalized: Slot,
    target: Slot,
) -> Result<bool, TransitionError> {
    let Some(index) = justified_index_after(target, finalized) else {
        return Ok(true);
    };
    bits.get(index).copied().ok_or_else(|| {
        TransitionError::JustifiedSlotOutOfRange(format!(
            "slot {target} outside tracked range (finalized={finalized}, len={})",
            bits.len()
        ))
    })
}

/// Extend the bitlist so `target` is addressable; new slots are false.
pub fn extend_to_slot(bits: &[bool], finalized: Slot, target: Slot) -> Vec<bool> {
    let Some(index) = justified_index_after(target, finalized) else {
        return bits.to_vec();
    };
    let required = index + 1;
    if required <= bits.len() {
        return bits.to_vec();
    }
    let mut out = bits.to_vec();
    out.resize(required, false);
    out
}
