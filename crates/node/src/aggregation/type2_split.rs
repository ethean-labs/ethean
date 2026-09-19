//! Seed the aggregate pool from a Type-2 SignedBlock (D5 known-payload recovery).

use crate::aggregation::{AggregatePool, PoolEntry, PoolKey};
use crate::gossip_pool::pool_profile_digest;
use ethean_crypto::{attestation_leaves_from_type2, Type1Leaf};
use ethean_transition::type2_statement_for_block;
use ethean_types::{AggregatedAttestation, SignedBlock};

/// How many pool keys were touched while reseeding from a block proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Type2SplitSeed {
    /// Attestation components listed on the Type-2 statement.
    pub leaves: usize,
    /// Pool entries inserted (one per body attestation with matching leaf).
    pub inserted: usize,
    /// True when leanVM returned real Type-1 proof bytes (always false today).
    pub crypto_split: bool,
}

/// Split the block's Type-2 proof structurally and insert Type-1 cache rows.
///
/// Uses attestation SSZ from the block body so the next proposer can rebuild
/// coverage without waiting for gossip re-delivery. Proof bytes stay empty until
/// leanVM exposes a real split.
pub fn seed_pool_from_signed_block(
    pool: &mut AggregatePool,
    signed: &SignedBlock,
) -> Type2SplitSeed {
    if signed.proof.proof.is_empty() {
        return Type2SplitSeed::default();
    }
    let Ok(statement) = type2_statement_for_block(&signed.block) else {
        return Type2SplitSeed::default();
    };
    let Ok(leaves) = attestation_leaves_from_type2(&statement, &signed.proof.proof) else {
        return Type2SplitSeed::default();
    };
    let mut out = Type2SplitSeed {
        leaves: leaves.len(),
        inserted: 0,
        crypto_split: leaves.iter().any(|l| l.crypto_split),
    };
    let profile = pool_profile_digest();
    for (att, leaf) in signed.block.body.attestations.iter().zip(leaves.iter()) {
        if att.data.hash_tree_root() != leaf.message_root {
            continue;
        }
        insert_leaf(pool, profile, att, leaf);
        out.inserted = out.inserted.saturating_add(1);
    }
    out
}

fn insert_leaf(
    pool: &mut AggregatePool,
    profile: [u8; 32],
    att: &AggregatedAttestation,
    leaf: &Type1Leaf,
) {
    let coverage = att.aggregation_bits.bits.iter().filter(|b| **b).count() as u32;
    pool.insert_verified(
        PoolKey {
            profile_digest: profile,
            message_root: leaf.message_root,
        },
        PoolEntry {
            proof: leaf.proof.clone(),
            coverage,
            inserted_slot: leaf.slot,
            attestation_ssz: att.ssz_encode(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, ValidatorIndex};
    use ethean_types::{
        AggregationBits, AttestationData, Block, BlockBody, Checkpoint, MultiMessageAggregate,
    };

    #[test]
    fn seeds_pool_from_attestation_components() {
        let data = AttestationData {
            slot: Slot::new(5),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        let att = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, false, true],
            },
            data,
        };
        let body = BlockBody::new(vec![att.clone()]).unwrap();
        let block = Block {
            slot: Slot::new(5),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            body,
        };
        let signed = SignedBlock::new(
            block,
            MultiMessageAggregate::new(vec![7u8; 32]).unwrap(),
        );
        let mut pool = AggregatePool::new(4);
        let seed = seed_pool_from_signed_block(&mut pool, &signed);
        assert_eq!(seed.leaves, 1);
        assert_eq!(seed.inserted, 1);
        assert!(!seed.crypto_split);
        assert_eq!(pool.len(), 1);
        let best = pool.best_entries();
        assert_eq!(best[0].1.coverage, 2);
        assert!(!best[0].1.attestation_ssz.is_empty());
    }

    #[test]
    fn empty_proof_is_noop() {
        let signed = SignedBlock::default();
        let mut pool = AggregatePool::new(2);
        assert_eq!(
            seed_pool_from_signed_block(&mut pool, &signed),
            Type2SplitSeed::default()
        );
        assert!(pool.is_empty());
    }
}
