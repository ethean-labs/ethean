//! Lean Consensus state transition (lstar / leanSpec@0b7d33ec).
//!
//! Structural path: [`apply_block_unverified`] / [`transition_block`] with
//! `require_proofs = false`. Verified path: [`apply_block`] checks the block's
//! merged leanMultisig proof against the parent registry, then transitions.

#![forbid(unsafe_code)]

mod block;
mod block_proof;
mod context;
mod error;
mod helpers;
mod operation;
mod opts;
mod outcome;
mod slot;

pub use block_proof::{block_proof_components, participant_indices, verify_block_proof};
pub use context::TransitionContext;
pub use error::TransitionError;
pub use opts::TransitionOpts;
pub use outcome::{TransitionOutcome, TransitionTimings};
pub use slot::process_slots;

pub use block::{process_block, process_block_header};
pub use helpers::{extend_to_slot, is_justifiable_after, is_slot_justified, proposer_for_slot};
pub use operation::lies_on_chain;
pub use operation::{
    check_attestation_data_structure, distinct_attestation_data_count, process_attestations,
};

use std::time::Instant;

use ethean_crypto::AggregateVerifier;
use ethean_types::{Block, SignedBlock, State};

/// Full state transition: advance slots to `block.slot`, process block, check state root.
///
/// When `opts.require_proofs` is true, returns [`TransitionError::UnsupportedSignature`]
/// (no proof is attached to a bare [`Block`]). Prefer [`apply_block`] for the verified API.
pub fn transition_block(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
    opts: TransitionOpts,
) -> Result<TransitionOutcome, TransitionError> {
    if opts.require_proofs {
        return Err(TransitionError::UnsupportedSignature(
            "require_proofs set but no aggregate proof provided; use apply_block(SignedBlock)"
                .into(),
        ));
    }
    apply_block_unverified(pre, block, ctx)
}

/// Structural transition for fixtures / tests (no XMSS, no proof required).
pub fn apply_block_unverified(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    let mut state = pre.clone();
    let started = Instant::now();
    let from_slot = state.slot.get();
    process_slots(&mut state, block.slot)?;
    let mut timings = TransitionTimings {
        slots_processed: block.slot.get().saturating_sub(from_slot),
        slots: started.elapsed(),
        ..TransitionTimings::default()
    };
    timed_block(&mut state, block, ctx, &mut timings)?;
    finish_unverified(state, block, timings)
}

/// `process_block` with the block and attestation phases timed.
fn timed_block(
    state: &mut State,
    block: &Block,
    ctx: &TransitionContext,
    timings: &mut TransitionTimings,
) -> Result<(), TransitionError> {
    let started = Instant::now();
    process_block_header(state, block)?;
    let attestations_started = Instant::now();
    process_attestations(state, &block.body.attestations, ctx)?;
    timings.attestations = attestations_started.elapsed();
    timings.attestations_processed = block.body.attestations.len() as u64;
    timings.block = started.elapsed();
    Ok(())
}

/// Structural apply at the current store slot (no `process_slots`).
///
/// Used by STF fixtures that intentionally skip slot processing (same-slot /
/// older-than-header rejection paths) and by runners remapping placeholder
/// zero state-roots onto `BLOCK_SLOT_MISMATCH` when `pre.slot != block.slot`.
pub fn apply_block_unverified_no_slots(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    let mut state = pre.clone();
    let mut timings = TransitionTimings::default();
    timed_block(&mut state, block, ctx, &mut timings)?;
    finish_unverified(state, block, timings)
}

fn finish_unverified(
    state: State,
    block: &Block,
    timings: TransitionTimings,
) -> Result<TransitionOutcome, TransitionError> {
    let post_state_root = state
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;
    if block.state_root != post_state_root {
        return Err(TransitionError::InvalidStateRoot(
            "Invalid block state root".into(),
        ));
    }
    Ok(TransitionOutcome {
        post_state: state,
        post_state_root,
        timings,
    })
}

/// Verified API: check the block's merged proof against the parent state's
/// registry (leanSpec `verify_signatures`), then apply the transition.
pub fn apply_block(
    pre: &State,
    signed: &SignedBlock,
    ctx: &TransitionContext,
    verifier: &dyn AggregateVerifier,
) -> Result<TransitionOutcome, TransitionError> {
    verify_block_proof(signed, &pre.validators, verifier)?;
    apply_block_unverified(pre, &signed.block, ctx)
}

/// Alias matching leanSpec naming: slots then block (structural / unverified).
pub fn state_transition(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    apply_block_unverified(pre, block, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_types::{
        AggregatedAttestation, AggregationBits, AttestationData, BlockBody, BlockHeader,
        Checkpoint, GenesisConfig, MultiMessageAggregate, Validator, MAX_ATTESTATIONS_DATA,
    };

    fn ctx() -> TransitionContext {
        TransitionContext::new(lstar_devnet().unwrap())
    }

    fn sample_state(validators: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..validators {
            vals.push(
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64))
                    .unwrap(),
            );
        }
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
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

    fn empty_block(slot: u64, proposer: u64, parent: ethean_primitives::Hash32) -> Block {
        Block {
            slot: Slot::new(slot),
            proposer_index: ValidatorIndex::new(proposer),
            parent_root: parent,
            state_root: HASH32_ZERO,
            body: BlockBody::default(),
        }
    }

    #[test]
    fn process_slots_advances() {
        let mut state = sample_state(2);
        process_slots(&mut state, Slot::new(4)).unwrap();
        assert_eq!(state.slot.get(), 4);
    }

    #[test]
    fn process_slots_rejects_past_or_equal() {
        let mut state = sample_state(1);
        state.slot = Slot::new(2);
        assert!(matches!(
            process_slots(&mut state, Slot::new(2)),
            Err(TransitionError::BlockSlotNotInFuture(_))
        ));
    }

    #[test]
    fn rejects_too_many_distinct_attestation_data() {
        let mut atts = Vec::new();
        for i in 0..(MAX_ATTESTATIONS_DATA + 1) {
            atts.push(AggregatedAttestation {
                aggregation_bits: AggregationBits { bits: vec![true] },
                data: AttestationData {
                    slot: Slot::new(i as u64),
                    head: Checkpoint::genesis(),
                    target: Checkpoint::genesis(),
                    source: Checkpoint::genesis(),
                },
            });
        }
        let mut state = sample_state(1);
        let body = BlockBody::new(atts).unwrap();
        // Cap check runs inside process_attestations regardless of chain filters.
        let err = crate::operation::process_attestations(&mut state, &body.attestations, &ctx())
            .unwrap_err();
        assert!(matches!(err, TransitionError::AttestationDataLimit(_)));
    }

    struct RejectAll;

    impl AggregateVerifier for RejectAll {
        fn verify_single(
            &self,
            _: &[u8],
            _: &[ethean_crypto::PublicKey],
            _: &[u8; 32],
            _: u64,
        ) -> ethean_crypto::Result<()> {
            Err(ethean_crypto::CryptoError::VerificationFailed)
        }
        fn verify_multi(
            &self,
            _: &[u8],
            _: &[ethean_crypto::ProofComponent],
        ) -> ethean_crypto::Result<()> {
            Err(ethean_crypto::CryptoError::VerificationFailed)
        }
    }

    #[test]
    fn apply_block_fails_closed_when_proof_is_rejected() {
        let state = sample_state(1);
        let signed = SignedBlock {
            block: empty_block(1, 0, HASH32_ZERO),
            proof: MultiMessageAggregate::default(),
        };
        let err = apply_block(&state, &signed, &ctx(), &RejectAll).unwrap_err();
        assert!(matches!(err, TransitionError::InvalidBlockProof(_)));
    }

    #[test]
    fn require_proofs_on_bare_block_rejected() {
        let state = sample_state(1);
        let block = empty_block(1, 0, HASH32_ZERO);
        let err =
            transition_block(&state, &block, &ctx(), TransitionOpts::REQUIRE_PROOFS).unwrap_err();
        assert!(matches!(err, TransitionError::UnsupportedSignature(_)));
    }

    #[test]
    fn structural_block_updates_header() {
        let pre = sample_state(3);
        // Parent root is hash_tree_root of the header after process_slots fills state_root.
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent_root = advanced.latest_block_header.hash_tree_root();
        // Proposer for slot 1 with 3 validators: 1 % 3 = 1
        let mut block = empty_block(1, 1, parent_root);

        let mut trial = pre.clone();
        process_slots(&mut trial, Slot::new(1)).unwrap();
        crate::block::process_block(&mut trial, &block, &ctx()).unwrap();
        block.state_root = trial.hash_tree_root().unwrap();

        let out = apply_block_unverified(&pre, &block, &ctx()).unwrap();
        assert_eq!(out.post_state.slot, Slot::new(1));
        assert_eq!(out.post_state.latest_block_header.slot, Slot::new(1));
        assert_eq!(out.post_state.latest_block_header.state_root, HASH32_ZERO);
    }
}
