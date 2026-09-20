//! Prefer Type-1 cache coverage when attaching a Type-2 block proof.

use crate::aggregation::{AggregatePool, PoolKey};
use crate::block_builder::type2::{try_attach_type2_proof, Type2ProveResult};
use crate::block_builder::PlanTransition;
use crate::gossip_pool::pool_profile_digest;

/// Outcome of a cache-aware Type-2 attach attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type2CacheAttach {
    /// Type-2 proof attached; `type1_hits` of `type1_needed` body atts had pool proofs.
    Attached {
        proof_len: usize,
        type1_hits: u32,
        type1_needed: u32,
    },
    /// Mesh path refused prove because some body atts lack Type-1 pool proofs.
    SkippedMissingCache {
        type1_hits: u32,
        type1_needed: u32,
    },
    /// Prove/verify unavailable or failed (plan unchanged).
    SkippedProve,
}

/// Count how many body attestations already have a non-empty Type-1 proof in `pool`.
pub fn type1_cache_stats(plan: &PlanTransition, pool: &AggregatePool) -> (u32, u32) {
    let profile = pool_profile_digest();
    let needed = plan.block.body.attestations.len() as u32;
    let mut hits = 0u32;
    for att in &plan.block.body.attestations {
        let key = PoolKey {
            profile_digest: profile,
            message_root: att.data.hash_tree_root(),
        };
        if pool.best(&key).is_some_and(|e| !e.proof.is_empty()) {
            hits = hits.saturating_add(1);
        }
    }
    (hits, needed)
}

/// Build Type-2 from the planned body, optionally requiring full Type-1 cache hits.
///
/// D3 mesh path (`require_type1_cache = true`): do not prove Type-2 until every packed
/// attestation has a Type-1 proof in the aggregate pool. Local-finality smoke passes
/// `false` so empty/injected bodies can still attach a statement-bound proof.
pub fn try_attach_type2_from_type1_cache(
    plan: &mut PlanTransition,
    pool: &AggregatePool,
    require_type1_cache: bool,
) -> Type2CacheAttach {
    let (type1_hits, type1_needed) = type1_cache_stats(plan, pool);
    if require_type1_cache && type1_needed > 0 && type1_hits < type1_needed {
        return Type2CacheAttach::SkippedMissingCache {
            type1_hits,
            type1_needed,
        };
    }
    match try_attach_type2_proof(plan) {
        Type2ProveResult::Attached { proof_len } => Type2CacheAttach::Attached {
            proof_len,
            type1_hits,
            type1_needed,
        },
        Type2ProveResult::Skipped => Type2CacheAttach::SkippedProve,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::PoolEntry;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{
        AggregatedAttestation, AggregationBits, AttestationData, Block, BlockBody, Checkpoint,
    };

    fn sample_att() -> AggregatedAttestation {
        AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true],
            },
            data: AttestationData {
                slot: Slot::new(3),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        }
    }

    fn plan_with_att(att: &AggregatedAttestation) -> PlanTransition {
        PlanTransition {
            parent_root: [1u8; 32],
            block: Block {
                slot: Slot::new(4),
                proposer_index: ValidatorIndex::new(0),
                parent_root: [1u8; 32],
                state_root: [3u8; 32],
                body: BlockBody::new(vec![att.clone()]).unwrap(),
            },
            aggregate_proof: Vec::new(),
            proposer_signature: None,
        }
    }

    #[test]
    fn require_cache_skips_without_type1_proof() {
        let att = sample_att();
        let mut plan = plan_with_att(&att);
        let pool = AggregatePool::default();
        assert_eq!(
            try_attach_type2_from_type1_cache(&mut plan, &pool, true),
            Type2CacheAttach::SkippedMissingCache {
                type1_hits: 0,
                type1_needed: 1,
            }
        );
        assert!(plan.aggregate_proof.is_empty());
    }

    #[test]
    fn require_cache_attaches_when_type1_present() {
        let att = sample_att();
        let mut plan = plan_with_att(&att);
        let mut pool = AggregatePool::default();
        pool.insert_verified(
            PoolKey {
                profile_digest: pool_profile_digest(),
                message_root: att.data.hash_tree_root(),
            },
            PoolEntry {
                proof: vec![1, 2, 3],
                coverage: 2,
                inserted_slot: 3,
                attestation_ssz: att.ssz_encode(),
            },
        );
        match try_attach_type2_from_type1_cache(&mut plan, &pool, true) {
            Type2CacheAttach::Attached {
                type1_hits,
                type1_needed,
                ..
            } => {
                assert_eq!((type1_hits, type1_needed), (1, 1));
            }
            Type2CacheAttach::SkippedProve => {
                assert!(plan.aggregate_proof.is_empty());
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
