//! Lean Consensus state transition (lstar / leanSpec@0b7d33ec).
//!
//! Structural path: [`apply_block_unverified`] / [`transition_block`] with
//! `require_proofs = false`. Verified path: [`apply_block`] — rejects empty
//! proofs and returns [`TransitionError::UnsupportedSignature`] until Phase 07/08
//! XMSS verification lands (never fake-accepts).

#![forbid(unsafe_code)]

mod block;
mod context;
mod error;
mod helpers;
mod operation;
mod opts;
mod outcome;
mod slot;

pub use context::TransitionContext;
pub use error::TransitionError;
pub use opts::TransitionOpts;
pub use outcome::TransitionOutcome;
pub use slot::process_slots;

use ethean_types::{Block, SignedBlock, State};

use block::process_block;

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
    process_slots(&mut state, block.slot)?;
    process_block(&mut state, block, ctx)?;
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
    })
}

/// Verified API: requires a non-empty multi-message proof; XMSS verify deferred to Phase 07/08.
pub fn apply_block(
    pre: &State,
    signed: &SignedBlock,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    if signed.proof.proof.is_empty() {
        return Err(TransitionError::UnsupportedSignature(
            "block aggregate proof is empty".into(),
        ));
    }
    let _ = (pre, ctx);
    Err(TransitionError::UnsupportedSignature(
        "XMSS / leanMultisig verification not implemented (Phase 07/08)".into(),
    ))
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
                aggregation_bits: AggregationBits {
                    bits: vec![true],
                },
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

    #[test]
    fn apply_block_rejects_empty_proof() {
        let state = sample_state(1);
        let signed = SignedBlock {
            block: empty_block(1, 0, HASH32_ZERO),
            proof: MultiMessageAggregate::default(),
        };
        let err = apply_block(&state, &signed, &ctx()).unwrap_err();
        assert!(matches!(err, TransitionError::UnsupportedSignature(_)));
    }

    #[test]
    fn require_proofs_on_bare_block_rejected() {
        let state = sample_state(1);
        let block = empty_block(1, 0, HASH32_ZERO);
        let err = transition_block(&state, &block, &ctx(), TransitionOpts::REQUIRE_PROOFS)
            .unwrap_err();
        assert!(matches!(err, TransitionError::UnsupportedSignature(_)));
    }

    #[test]
    fn structural_block_updates_header() {
        let mut pre = sample_state(3);
        // Cache parent header root after filling state_root via process_slots path.
        let parent_root = {
            let mut s = pre.clone();
            process_slots(&mut s, Slot::new(1)).unwrap();
            // After slots, header still points at genesis; parent for block is that header root.
            s.latest_block_header.hash_tree_root()
        };
        // Rebuild: process_slots mutates header state_root; parent_root must match post-slots header.
        process_slots(&mut pre, Slot::new(1)).unwrap();
        let parent_root = pre.latest_block_header.hash_tree_root();
        let mut block = empty_block(1, 1 % 3, parent_root);
        // Proposer for slot 1 with 3 validators: 1 % 3 = 1
        block.proposer_index = ValidatorIndex::new(1);

        // Compute expected post-state root by dry-run without state_root check.
        let mut trial = pre.clone();
        process_block(&mut trial, &block, &ctx()).unwrap();
        block.state_root = trial.hash_tree_root().unwrap();

        let out = apply_block_unverified(&pre, &block, &ctx()).unwrap();
        assert_eq!(out.post_state.slot, Slot::new(1));
        assert_eq!(out.post_state.latest_block_header.slot, Slot::new(1));
        assert_eq!(out.post_state.latest_block_header.state_root, HASH32_ZERO);
        let _ = parent_root;
    }
}
