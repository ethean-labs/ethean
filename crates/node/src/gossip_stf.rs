//! Verified import of decoded `SignedBlock`s (leanSpec `on_block` signature
//! step followed by the state transition).

use crate::chain_owner::ChainOwner;
use crate::gossip_decode::DecodedBlockGossip;
use crate::lean_metrics;
use crate::shutdown::{ShutdownPhase, ShutdownState};
use ethean_metrics::lean::observe_since;
use ethean_multisig::LeanMultisigVerifier;
use ethean_primitives::Hash32;
use ethean_transition::{apply_block, TransitionContext};
use std::time::Instant;

/// Outcome of importing a decoded block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GossipStfResult {
    /// Does not extend the head, no local state yet, or shutting down.
    Skipped,
    /// Proof verified and transition applied; head advanced.
    Applied { root: Hash32 },
    /// Proof or transition rejected; head unchanged.
    Rejected { reason: String },
}

/// Verify the block proof against the parent registry, then transition.
pub fn import_decoded_block(
    owner: &mut ChainOwner,
    shutdown: &ShutdownState,
    decoded: &DecodedBlockGossip,
) -> GossipStfResult {
    if shutdown.phase() == ShutdownPhase::Stopped || decoded.parent != owner.head_root {
        return GossipStfResult::Skipped;
    }
    let (Some(pre), Some(profile)) = (owner.head_state.as_ref(), owner.profile.clone()) else {
        return GossipStfResult::Skipped;
    };
    let ctx = TransitionContext::new(profile);
    let started = Instant::now();
    let applied = apply_block(pre, &decoded.signed, &ctx, &LeanMultisigVerifier);
    observe_since(
        "lean_fork_choice_block_processing_time_seconds",
        &[],
        started,
    );
    match applied {
        Ok(out) => {
            lean_metrics::transition(started.elapsed(), &out.timings);
            owner.head_state = Some(out.post_state);
            owner.advance_head(decoded.root, decoded.parent);
            GossipStfResult::Applied { root: decoded.root }
        }
        Err(e) => GossipStfResult::Rejected {
            reason: e.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip_decode::try_decode_block;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_profile::lstar_devnet;
    use ethean_types::{
        Block, BlockBody, BlockHeader, Checkpoint, GenesisConfig, MultiMessageAggregate,
        SignedBlock, State, Validator,
    };

    fn owner_with_state() -> ChainOwner {
        let state = State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap(),
            ],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        };
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        owner.head_state = Some(state);
        owner.profile = Some(lstar_devnet().unwrap());
        owner
    }

    fn decoded(parent: Hash32, proof: Vec<u8>) -> DecodedBlockGossip {
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: parent,
            state_root: [0u8; 32],
            body: BlockBody::default(),
        };
        let signed = SignedBlock::new(block, MultiMessageAggregate::new(proof).unwrap());
        try_decode_block("/x/block/y", &signed.ssz_encode().unwrap()).unwrap()
    }

    #[test]
    fn rejects_blocks_whose_proof_does_not_verify() {
        let mut owner = owner_with_state();
        let head = owner.head_root;
        let result = import_decoded_block(
            &mut owner,
            &ShutdownState::default(),
            &decoded(head, vec![0u8; 32]),
        );
        assert!(
            matches!(result, GossipStfResult::Rejected { .. }),
            "{result:?}"
        );
        assert_eq!(owner.head_root, head, "head unchanged");
    }

    #[test]
    fn skips_non_extending_blocks_and_missing_state() {
        let mut owner = owner_with_state();
        let result = import_decoded_block(
            &mut owner,
            &ShutdownState::default(),
            &decoded([9u8; 32], vec![1]),
        );
        assert_eq!(result, GossipStfResult::Skipped);
        owner.head_state = None;
        let head = owner.head_root;
        let result = import_decoded_block(
            &mut owner,
            &ShutdownState::default(),
            &decoded(head, vec![1]),
        );
        assert_eq!(
            result,
            GossipStfResult::Skipped,
            "never advance head unverified"
        );
    }
}
