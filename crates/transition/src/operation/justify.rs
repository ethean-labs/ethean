//! 3SF-mini justification and finalization from aggregated attestations.

use std::collections::HashMap;

use ethean_primitives::{Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_types::{AggregatedAttestation, Checkpoint, State};

use crate::error::TransitionError;
use crate::helpers::{
    index_within_registry, is_justifiable_after, is_slot_justified, justified_index_after,
};

/// Whether attestation checkpoints lie on the historical chain (leanSpec `lies_on_chain`).
pub fn lies_on_chain(data: &ethean_types::AttestationData, historical: &[Hash32]) -> bool {
    if data.source.root == HASH32_ZERO
        || data.target.root == HASH32_ZERO
        || data.head.root == HASH32_ZERO
    {
        return false;
    }
    let src = data.source.slot.get() as usize;
    let tgt = data.target.slot.get() as usize;
    let head = data.head.slot.get() as usize;
    if src >= historical.len() || tgt >= historical.len() || head >= historical.len() {
        return false;
    }
    historical[src] == data.source.root
        && historical[tgt] == data.target.root
        && historical[head] == data.head.root
}

fn validator_indices_from_bits(bits: &[bool]) -> Result<Vec<ValidatorIndex>, TransitionError> {
    let indices: Vec<_> = bits
        .iter()
        .enumerate()
        .filter_map(|(i, b)| b.then_some(ValidatorIndex::new(i as u64)))
        .collect();
    if indices.is_empty() {
        return Err(TransitionError::EmptyAggregationBits(
            "Aggregated attestation must reference at least one validator".into(),
        ));
    }
    Ok(indices)
}

/// Core justification update (leanSpec `process_attestations` body after the data-cap check).
pub fn apply_justifications(
    state: &mut State,
    attestations: &[AggregatedAttestation],
) -> Result<(), TransitionError> {
    let validator_count = state.validators.len();
    if validator_count == 0 {
        return Err(TransitionError::EmptyValidatorRegistry(
            "State holds no validators to segment justification votes against".into(),
        ));
    }

    let expected_votes = state.justifications_roots.len() * validator_count;
    if state.justifications_validators.len() != expected_votes {
        return Err(TransitionError::JustificationVotesLengthMismatch(
            "Justification vote list length does not equal tracked-root count times validator count"
                .into(),
        ));
    }

    if state.justifications_roots.iter().any(|r| *r == HASH32_ZERO) {
        return Err(TransitionError::ZeroHashJustificationRoot(
            "Tracked justification roots contain the zero hash".into(),
        ));
    }

    let mut justifications: HashMap<Hash32, Vec<bool>> = HashMap::new();
    for (i, root) in state.justifications_roots.iter().enumerate() {
        let start = i * validator_count;
        let end = start + validator_count;
        justifications.insert(*root, state.justifications_validators[start..end].to_vec());
    }

    let mut latest_justified = state.latest_justified;
    let mut latest_finalized = state.latest_finalized;
    let mut finalized_slot = latest_finalized.slot;
    let mut justified_slots = state.justified_slots.clone();

    let start_slot = (finalized_slot.get() + 1) as usize;
    let mut root_to_slot: HashMap<Hash32, Slot> = HashMap::new();
    for (i, root) in state
        .historical_block_hashes
        .iter()
        .enumerate()
        .skip(start_slot)
    {
        root_to_slot.insert(*root, Slot::new(i as u64));
    }

    for attestation in attestations {
        apply_one_vote(
            attestation,
            state,
            validator_count,
            &mut justifications,
            &mut latest_justified,
            &mut latest_finalized,
            &mut finalized_slot,
            &mut justified_slots,
            &mut root_to_slot,
        )?;
    }

    let mut sorted_roots: Vec<Hash32> = justifications.keys().copied().collect();
    sorted_roots.sort_unstable();
    let mut flat = Vec::with_capacity(sorted_roots.len() * validator_count);
    for root in &sorted_roots {
        flat.extend_from_slice(justifications.get(root).unwrap());
    }

    state.justifications_roots = sorted_roots;
    state.justifications_validators = flat;
    state.justified_slots = justified_slots;
    state.latest_justified = latest_justified;
    state.latest_finalized = latest_finalized;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn apply_one_vote(
    attestation: &AggregatedAttestation,
    state: &State,
    validator_count: usize,
    justifications: &mut HashMap<Hash32, Vec<bool>>,
    latest_justified: &mut Checkpoint,
    latest_finalized: &mut Checkpoint,
    finalized_slot: &mut Slot,
    justified_slots: &mut Vec<bool>,
    root_to_slot: &mut HashMap<Hash32, Slot>,
) -> Result<(), TransitionError> {
    let source = attestation.data.source;
    let target = attestation.data.target;

    if !is_slot_justified(justified_slots, *finalized_slot, source.slot)? {
        return Ok(());
    }
    if is_slot_justified(justified_slots, *finalized_slot, target.slot)? {
        return Ok(());
    }
    if !lies_on_chain(&attestation.data, &state.historical_block_hashes) {
        return Ok(());
    }
    if target.slot.get() <= source.slot.get() {
        return Ok(());
    }
    if !is_justifiable_after(target.slot, *finalized_slot) {
        return Ok(());
    }

    let voting = validator_indices_from_bits(&attestation.aggregation_bits.bits)?;
    for vi in &voting {
        if !index_within_registry(*vi, validator_count as u64) {
            return Err(TransitionError::ValidatorIndexOutOfRange(
                "Attestation aggregation bits reference a validator outside the registry".into(),
            ));
        }
    }

    justifications
        .entry(target.root)
        .or_insert_with(|| vec![false; validator_count]);
    let tally = justifications.get_mut(&target.root).unwrap();
    for vi in &voting {
        tally[vi.as_usize()] = true;
    }

    let count = tally.iter().filter(|b| **b).count();
    if 3 * count < 2 * validator_count {
        return Ok(());
    }

    if target.slot > latest_justified.slot {
        *latest_justified = target;
    }

    let justified_index = justified_index_after(target.slot, *finalized_slot)
        .expect("justified target must have an index after finalized");
    if justified_index >= justified_slots.len() {
        justified_slots.resize(justified_index + 1, false);
    }
    justified_slots[justified_index] = true;
    justifications.remove(&target.root);

    if source.slot > *finalized_slot {
        let mut gap_justifiable = false;
        let mut s = source.slot.get() + 1;
        while s < target.slot.get() {
            if is_justifiable_after(Slot::new(s), *finalized_slot) {
                gap_justifiable = true;
                break;
            }
            s += 1;
        }
        if !gap_justifiable {
            let old_finalized = *finalized_slot;
            *latest_finalized = source;
            *finalized_slot = latest_finalized.slot;
            let delta = (finalized_slot.get() - old_finalized.get()) as usize;
            if delta > 0 {
                if delta >= justified_slots.len() {
                    justified_slots.clear();
                } else {
                    *justified_slots = justified_slots[delta..].to_vec();
                }
                justifications.retain(|root, _| {
                    root_to_slot
                        .get(root)
                        .map(|slot| *slot > *finalized_slot)
                        .unwrap_or(false)
                });
            }
        }
    }
    Ok(())
}
