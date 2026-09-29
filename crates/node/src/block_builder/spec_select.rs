//! leanSpec `build_block` attestation selection (`lstar/block_production.py`).
//!
//! Candidates are scanned in target-slot order and accepted only when the
//! proposer knows the head block, the source is the current justified
//! checkpoint, the checkpoints lie on this chain, the source slot is
//! justified and the target is not yet justified (genesis self-votes are
//! kept for their head weight). Accepting votes can justify new sources, so
//! the scan repeats until a pass adds nothing. One proof is carried per data:
//! the variant covering the most validators (the spec folds several proofs
//! into a child merge; Ethean leaves that union to the aggregator round).

use std::collections::HashSet;

use ethean_primitives::{Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_profile::ChainProfile;
use ethean_transition::{
    extend_to_slot, is_slot_justified, lies_on_chain, process_block, process_slots,
    TransitionContext,
};
use ethean_types::{
    AggregatedAttestation, AggregationBits, AttestationData, Block, BlockBody, Checkpoint, State,
    MAX_ATTESTATIONS_DATA,
};

use super::attestations::ProofVariant;

/// Selected body attestations and the Type-1 proof of each (parallel lists).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectedBody {
    pub attestations: Vec<AggregatedAttestation>,
    pub proofs: Vec<Vec<u8>>,
}

/// Chain view once this block applies: history, the parent, then skipped slots.
pub fn extended_chain_view(pre: &State, parent_root: Hash32, slot: Slot) -> Vec<Hash32> {
    let mut view = pre.historical_block_hashes.clone();
    view.push(parent_root);
    let parent_slot = pre.latest_block_header.slot.get();
    let empty = slot.get().saturating_sub(parent_slot + 1);
    view.extend(std::iter::repeat_n(HASH32_ZERO, empty as usize));
    view
}

/// The proof variant covering the most validators; ties go to the larger encoding.
pub fn best_variant(variants: &[ProofVariant]) -> Option<&ProofVariant> {
    variants.iter().max_by(|a, b| {
        a.coverage()
            .cmp(&b.coverage())
            .then_with(|| a.proof.cmp(&b.proof))
    })
}

fn justified(bits: &[bool], finalized: Slot, slot: Slot) -> bool {
    is_slot_justified(bits, finalized, slot).unwrap_or(false)
}

/// Equal justification bookkeeping (the header's body root always differs).
fn same_vote_state(a: &State, b: &State) -> bool {
    a.latest_justified == b.latest_justified
        && a.latest_finalized == b.latest_finalized
        && a.justified_slots == b.justified_slots
        && a.justifications_roots == b.justifications_roots
        && a.justifications_validators == b.justifications_validators
}

fn trial_post_state(
    advanced: &State,
    slot: Slot,
    proposer: ValidatorIndex,
    parent_root: Hash32,
    attestations: &[AggregatedAttestation],
    ctx: &TransitionContext,
) -> Result<State, String> {
    let body = BlockBody::new(attestations.to_vec()).map_err(|e| e.to_string())?;
    let block = Block {
        slot,
        proposer_index: proposer,
        parent_root,
        state_root: HASH32_ZERO,
        body,
    };
    let mut post = advanced.clone();
    process_block(&mut post, &block, ctx).map_err(|e| e.to_string())?;
    Ok(post)
}

/// Select body attestations from `candidates` following the spec rules.
pub fn select_body(
    candidates: &[(AttestationData, Vec<ProofVariant>)],
    pre: &State,
    slot: Slot,
    proposer: ValidatorIndex,
    parent_root: Hash32,
    known_roots: &HashSet<Hash32>,
    profile: ChainProfile,
) -> Result<SelectedBody, String> {
    select_body_capped(
        candidates,
        pre,
        slot,
        proposer,
        parent_root,
        known_roots,
        profile,
        MAX_ATTESTATIONS_DATA,
    )
}

/// [`select_body`] keeping at most `max_data` attestations.
///
/// Below the spec maximum, votes that leave the post-state unchanged (every
/// participant already counted for that target) are skipped so stale pool
/// data cannot fill the budget ahead of votes that still count.
#[allow(clippy::too_many_arguments)]
pub fn select_body_capped(
    candidates: &[(AttestationData, Vec<ProofVariant>)],
    pre: &State,
    slot: Slot,
    proposer: ValidatorIndex,
    parent_root: Hash32,
    known_roots: &HashSet<Hash32>,
    profile: ChainProfile,
    max_data: usize,
) -> Result<SelectedBody, String> {
    let mut out = SelectedBody::default();
    let cap = max_data.min(MAX_ATTESTATIONS_DATA);
    if candidates.is_empty() || cap == 0 {
        return Ok(out);
    }
    let skip_noops = cap < MAX_ATTESTATIONS_DATA;
    let ctx = TransitionContext::new(profile);
    let mut advanced = pre.clone();
    process_slots(&mut advanced, slot).map_err(|e| e.to_string())?;
    let mut current = if skip_noops {
        Some(trial_post_state(
            &advanced,
            slot,
            proposer,
            parent_root,
            &[],
            &ctx,
        )?)
    } else {
        None
    };

    // Genesis parents are justified at slot 0 by header processing.
    let mut justified_cp = if pre.latest_block_header.slot == Slot::ZERO {
        Checkpoint::new(parent_root, Slot::ZERO)
    } else {
        pre.latest_justified
    };
    let mut finalized_slot = pre.latest_finalized.slot;
    let mut justified_bits = extend_to_slot(
        &pre.justified_slots,
        finalized_slot,
        Slot::new(slot.get().saturating_sub(1)),
    );
    let chain = extended_chain_view(pre, parent_root, slot);

    let mut ordered: Vec<&(AttestationData, Vec<ProofVariant>)> = candidates.iter().collect();
    // Under a cap, equal target slots go widest first so the budget carries the most votes.
    let coverage = |c: &(AttestationData, Vec<ProofVariant>)| {
        if skip_noops {
            best_variant(&c.1).map(|v| v.coverage()).unwrap_or(0)
        } else {
            0
        }
    };
    ordered.sort_by(|a, b| {
        a.0.target
            .slot
            .cmp(&b.0.target.slot)
            .then_with(|| coverage(b).cmp(&coverage(a)))
            .then_with(|| a.0.hash_tree_root().cmp(&b.0.hash_tree_root()))
    });
    let mut processed: HashSet<Hash32> = HashSet::new();
    let mut skips = SelectionSkips::default();

    loop {
        let before = out.attestations.len();
        for (data, variants) in &ordered {
            let root = data.hash_tree_root();
            if processed.contains(&root) {
                continue;
            }
            if out.attestations.len() >= cap
                || (!skip_noops && processed.len() >= MAX_ATTESTATIONS_DATA)
            {
                break;
            }
            if !known_roots.contains(&data.head.root) {
                skips.unknown_head += 1;
                continue;
            }
            if data.source.slot != justified_cp.slot {
                skips.other_source += 1;
                continue;
            }
            if !lies_on_chain(data, &chain) {
                skips.off_chain += 1;
                continue;
            }
            if !justified(&justified_bits, finalized_slot, data.source.slot) {
                skips.other_source += 1;
                continue;
            }
            let genesis_self_vote =
                data.source.slot == Slot::ZERO && data.target.slot == Slot::ZERO;
            if !genesis_self_vote && justified(&justified_bits, finalized_slot, data.target.slot) {
                skips.target_justified += 1;
                continue;
            }
            let Some(best) = best_variant(variants) else {
                continue;
            };
            let bits = AggregationBits::new(best.bits.clone()).map_err(|e| e.to_string())?;
            let attestation = AggregatedAttestation {
                aggregation_bits: bits,
                data: *data,
            };
            // Keep the body valid: drop a vote the transition refuses.
            let mut trial = out.attestations.clone();
            trial.push(attestation.clone());
            processed.insert(root);
            let Ok(post) = trial_post_state(&advanced, slot, proposer, parent_root, &trial, &ctx)
            else {
                skips.refused += 1;
                continue;
            };
            if let Some(cur) = current.as_mut() {
                if same_vote_state(cur, &post) {
                    skips.no_effect += 1;
                    continue;
                }
                *cur = post;
            }
            out.attestations.push(attestation);
            out.proofs.push(best.proof.clone());
        }
        if out.attestations.len() == before {
            break;
        }
        let post = trial_post_state(
            &advanced,
            slot,
            proposer,
            parent_root,
            &out.attestations,
            &ctx,
        )?;
        if post.latest_justified != justified_cp || post.latest_finalized.slot != finalized_slot {
            justified_cp = post.latest_justified;
            justified_bits = post.justified_slots.clone();
            finalized_slot = post.latest_finalized.slot;
            continue;
        }
        break;
    }
    tracing::debug!(
        slot = slot.get(),
        justified = justified_cp.slot.get(),
        unknown_head = skips.unknown_head,
        other_source = skips.other_source,
        off_chain = skips.off_chain,
        target_justified = skips.target_justified,
        refused = skips.refused,
        no_effect = skips.no_effect,
        "block vote selection skips"
    );
    Ok(out)
}

/// Why candidates were left out of a body (debug visibility only).
#[derive(Debug, Default)]
struct SelectionSkips {
    unknown_head: u32,
    other_source: u32,
    off_chain: u32,
    target_justified: u32,
    refused: u32,
    no_effect: u32,
}

#[cfg(test)]
#[path = "spec_select_tests.rs"]
mod tests;
