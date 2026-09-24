//! Transition planning hook before proposal signing.

use crate::aggregation::AggregatePool;
use ethean_primitives::{Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_profile::ChainProfile;
use ethean_transition::{process_block, process_slots, TransitionContext};
use ethean_types::{Block, BlockBody, State};

use std::collections::HashSet;

use super::attestations::candidates_from_pool;
use super::spec_select::select_body;

/// Planned transition inputs for computing the post-state root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTransition {
    /// Parent root the block extends.
    pub parent_root: Hash32,
    /// Block with computed state root after structural transition.
    pub block: Block,
    /// Merged Type-2 block proof (empty until the prover returns it).
    pub aggregate_proof: Vec<u8>,
    /// Type-1 proof of each body attestation, parallel to `block.body.attestations`.
    pub attestation_proofs: Vec<Vec<u8>>,
}

impl PlanTransition {
    /// Block signing / tree root after state-root binding.
    pub fn block_root(&self) -> Result<Hash32, String> {
        self.block.hash_tree_root().map_err(|e| e.to_string())
    }

    /// Declared post-state root on the planned block.
    pub fn expected_state_root(&self) -> Hash32 {
        self.block.state_root
    }
}

/// Plan a proposal from the aggregate pool with leanSpec vote selection.
///
/// `known_roots` are the block roots this node has seen; votes for other
/// heads are left out (leanSpec `build_block` `known_block_roots`).
pub fn plan_from_pool(
    pool: &AggregatePool,
    parent_root: Hash32,
    slot: Slot,
    proposer_index: ValidatorIndex,
    pre: &State,
    profile: ChainProfile,
    known_roots: &HashSet<Hash32>,
) -> Result<PlanTransition, String> {
    let candidates = candidates_from_pool(pool);
    let selected = select_body(
        &candidates,
        pre,
        slot,
        proposer_index,
        parent_root,
        known_roots,
        profile.clone(),
    )?;
    let body = BlockBody::new(selected.attestations).map_err(|e| e.to_string())?;
    let mut plan = plan_with_body(parent_root, slot, proposer_index, body, pre, profile)?;
    plan.attestation_proofs = selected.proofs;
    Ok(plan)
}

fn plan_with_body(
    parent_root: Hash32,
    slot: Slot,
    proposer_index: ValidatorIndex,
    body: BlockBody,
    pre: &State,
    profile: ChainProfile,
) -> Result<PlanTransition, String> {
    let mut block = Block {
        slot,
        proposer_index,
        parent_root,
        state_root: HASH32_ZERO,
        body,
    };
    let ctx = TransitionContext::new(profile);
    let mut trial = pre.clone();
    process_slots(&mut trial, slot).map_err(|e| e.to_string())?;
    process_block(&mut trial, &block, &ctx).map_err(|e| e.to_string())?;
    block.state_root = trial.hash_tree_root().map_err(|e| e.to_string())?;
    Ok(PlanTransition {
        parent_root,
        block,
        aggregate_proof: Vec::new(),
        attestation_proofs: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Bytes52;
    use ethean_profile::lstar_devnet;
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, Validator};

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

    #[test]
    fn plans_empty_body_with_state_root() {
        let pre = sample_state(3);
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent = advanced.latest_block_header.hash_tree_root();
        let pool = AggregatePool::default();
        let plan = plan_from_pool(
            &pool,
            parent,
            Slot::new(1),
            ValidatorIndex::new(1),
            &pre,
            lstar_devnet().unwrap(),
            &HashSet::new(),
        )
        .expect("plan");
        assert_eq!(plan.parent_root, parent);
        assert_ne!(plan.expected_state_root(), HASH32_ZERO);
        assert!(plan.block_root().is_ok());
        assert!(plan.aggregate_proof.is_empty());
    }
}
