//! Transition planning hook before proposal signing.

use crate::aggregation::AggregatePool;
use ethean_primitives::{Hash32, Slot, ValidatorIndex, HASH32_ZERO};
use ethean_profile::ChainProfile;
use ethean_transition::{process_block, process_slots, TransitionContext};
use ethean_types::{AggregatedAttestation, AttestationData, Block, BlockBody, State};

use std::collections::HashSet;

use super::attestations::{candidates_from_pool, ProofVariant};
use super::spec_select::select_body_capped;

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

/// Attestation data a proved block carries by default. The block proof merges
/// one component per data plus the proposer's; leanVM `e2592df4` merge time
/// steps with the next power of two of that count (about 1.7 s, 3.3 s, 5.6 s
/// and 11.5 s for 1, 2, 4 and 8 components on a 20-core dev machine), so the
/// spec maximum (8 data, 9 components) cannot be proved within a 4 s slot.
/// One data keeps the merge at two components; when voters agree on head and
/// target it carries almost every vote. A block proof longer than the slot
/// makes consecutive proposers build siblings, and finality stalls on the
/// missing justifiable-slot blocks.
pub const DEFAULT_MAX_BLOCK_ATTESTATION_DATA: usize = 1;

/// Plan a proposal from the aggregate pool with leanSpec vote selection.
///
/// `known_roots` are the block roots this node has seen; votes for other
/// heads are left out (leanSpec `build_block` `known_block_roots`). At most
/// `max_data` selected attestations are kept, in selection order. Every
/// prefix of the selection is a valid body: each vote was accepted against
/// the state produced by the votes before it.
#[allow(clippy::too_many_arguments)]
pub fn plan_from_pool(
    pool: &AggregatePool,
    parent_root: Hash32,
    slot: Slot,
    proposer_index: ValidatorIndex,
    pre: &State,
    profile: ChainProfile,
    known_roots: &HashSet<Hash32>,
    max_data: usize,
) -> Result<PlanTransition, String> {
    let candidates = candidates_from_pool(pool);
    let selected = select_body_capped(
        &candidates,
        pre,
        slot,
        proposer_index,
        parent_root,
        known_roots,
        profile.clone(),
        max_data,
    )?;
    tracing::debug!(
        slot = slot.get(),
        candidates = candidates.len(),
        selected = selected.attestations.len(),
        max_data,
        votes = %describe_votes(&selected.attestations),
        pool = %describe_candidates(&candidates),
        "block attestations selected"
    );
    let body = BlockBody::new(selected.attestations).map_err(|e| e.to_string())?;
    let mut plan = plan_with_body(parent_root, slot, proposer_index, body, pre, profile)?;
    plan.attestation_proofs = selected.proofs;
    Ok(plan)
}

/// `source->target:participants` per body attestation, for debug logs.
fn describe_votes(attestations: &[AggregatedAttestation]) -> String {
    attestations
        .iter()
        .map(|a| {
            let n = a.aggregation_bits.bits.iter().filter(|b| **b).count();
            format!(
                "{}->{}:{n}",
                a.data.source.slot.get(),
                a.data.target.slot.get()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// `slot@source->target:best_coverage` per pool candidate, for debug logs.
fn describe_candidates(candidates: &[(AttestationData, Vec<ProofVariant>)]) -> String {
    candidates
        .iter()
        .map(|(d, variants)| {
            let n = variants.iter().map(|v| v.coverage()).max().unwrap_or(0);
            format!(
                "{}@{}->{}:{n}",
                d.slot.get(),
                d.source.slot.get(),
                d.target.slot.get()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
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
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, Validator, MAX_ATTESTATIONS_DATA};

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
            MAX_ATTESTATIONS_DATA,
        )
        .expect("plan");
        assert_eq!(plan.parent_root, parent);
        assert_ne!(plan.expected_state_root(), HASH32_ZERO);
        assert!(plan.block_root().is_ok());
        assert!(plan.aggregate_proof.is_empty());
    }

    #[test]
    fn capped_plans_leave_out_votes_that_change_nothing() {
        use crate::aggregation::PoolEntry;
        use crate::gossip_attestation::pool_key;
        use ethean_types::{AggregationBits, AttestationData};

        let pre = sample_state(4);
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent = advanced.latest_block_header.hash_tree_root();
        let genesis = Checkpoint::new(parent, Slot::ZERO);
        let mut pool = AggregatePool::default();
        for vote_slot in [0u64, 1] {
            let data = AttestationData {
                slot: Slot::new(vote_slot),
                head: genesis,
                target: genesis,
                source: genesis,
            };
            let attestation = AggregatedAttestation {
                aggregation_bits: AggregationBits::new(vec![true, true, true, false]).unwrap(),
                data,
            };
            pool.insert_verified(
                pool_key(data.hash_tree_root()),
                PoolEntry {
                    proof: vec![vote_slot as u8 + 1; 4],
                    coverage: 3,
                    inserted_slot: vote_slot,
                    attestation_ssz: attestation.ssz_encode(),
                },
            );
        }
        let known: HashSet<Hash32> = [parent].into_iter().collect();
        let plan_with = |cap| {
            plan_from_pool(
                &pool,
                parent,
                Slot::new(1),
                ValidatorIndex::new(1),
                &pre,
                lstar_devnet().unwrap(),
                &known,
                cap,
            )
            .expect("plan")
        };
        let full = plan_with(MAX_ATTESTATIONS_DATA);
        assert_eq!(
            full.block.body.attestations.len(),
            2,
            "the spec selection keeps genesis self-votes"
        );
        let capped = plan_with(1);
        assert!(capped.block.body.attestations.is_empty());
        assert!(capped.attestation_proofs.is_empty());
        assert!(plan_with(0).block.body.attestations.is_empty());
    }
}
