//! Coalesce aggregate-pool variants for one message via Type-1 merge.

use crate::aggregation::{AggregatePool, PoolEntry, PoolKey};
use ethean_crypto::{merge_type1, AggregateStatement, ParticipantSet, ProofKind};
use ethean_types::{AggregatedAttestation, AggregationBits};

/// Outcome of attempting to merge the two best pool variants for a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolMergeResult {
    /// Need at least two variants with non-empty proofs.
    Skipped,
    /// Merged proof inserted; reports new coverage.
    Merged { coverage: u32 },
    /// Crypto merge failed (backend unavailable or verify failed).
    Failed(String),
}

/// Merge the two highest-coverage variants for `key` into one Type-1 proof.
///
/// Requires each variant to carry non-empty proof bytes and decodable attestation SSZ.
pub fn merge_best_pool_variants(pool: &mut AggregatePool, key: PoolKey) -> PoolMergeResult {
    let Some(list) = pool.variants(&key) else {
        return PoolMergeResult::Skipped;
    };
    if list.len() < 2 {
        return PoolMergeResult::Skipped;
    }
    let mut ranked = list;
    ranked.sort_by(|a, b| {
        b.coverage
            .cmp(&a.coverage)
            .then_with(|| a.inserted_slot.cmp(&b.inserted_slot))
    });
    let left = &ranked[0];
    let right = &ranked[1];
    if left.proof.is_empty() || right.proof.is_empty() {
        return PoolMergeResult::Skipped;
    }
    let (Ok(left_att), Ok(right_att)) = (
        AggregatedAttestation::ssz_decode(&left.attestation_ssz),
        AggregatedAttestation::ssz_decode(&right.attestation_ssz),
    ) else {
        return PoolMergeResult::Skipped;
    };
    let left_stmt = match statement_from_attestation(key.profile_digest, &left_att) {
        Ok(s) => s,
        Err(e) => return PoolMergeResult::Failed(e),
    };
    let right_stmt = match statement_from_attestation(key.profile_digest, &right_att) {
        Ok(s) => s,
        Err(e) => return PoolMergeResult::Failed(e),
    };
    match merge_type1(&left_stmt, &left.proof, &right_stmt, &right.proof) {
        Ok((merged_stmt, proof)) => {
            let bits = participants_to_bits(merged_stmt.participants.as_slice());
            let coverage = bits.iter().filter(|b| **b).count() as u32;
            let att = AggregatedAttestation {
                aggregation_bits: AggregationBits { bits },
                data: left_att.data.clone(),
            };
            pool.insert_verified(
                key,
                PoolEntry {
                    proof,
                    coverage,
                    inserted_slot: left.inserted_slot.max(right.inserted_slot),
                    attestation_ssz: att.ssz_encode(),
                },
            );
            PoolMergeResult::Merged { coverage }
        }
        Err(e) => PoolMergeResult::Failed(e.to_string()),
    }
}

fn statement_from_attestation(
    profile: [u8; 32],
    att: &AggregatedAttestation,
) -> Result<AggregateStatement, String> {
    let mut indices = Vec::new();
    for (i, bit) in att.aggregation_bits.bits.iter().enumerate() {
        if *bit {
            indices.push(i as u32);
        }
    }
    let participants = ParticipantSet::try_from_ordered(indices).map_err(|e| e.to_string())?;
    Ok(AggregateStatement {
        kind: ProofKind::Type1,
        profile_digest: profile,
        message_root: att.data.hash_tree_root(),
        slot: att.data.slot.get(),
        participants,
        components: vec![],
    })
}

fn participants_to_bits(indices: &[u32]) -> Vec<bool> {
    let max = indices.last().map(|i| *i as usize + 1).unwrap_or(0);
    let mut bits = vec![false; max];
    for &i in indices {
        if let Some(slot) = bits.get_mut(i as usize) {
            *slot = true;
        }
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip_pool::pool_profile_digest;
    use ethean_crypto::prove_type1;
    use ethean_primitives::Slot;
    use ethean_types::{AttestationData, Checkpoint};

    fn entry(bits: Vec<bool>, slot: u64) -> PoolEntry {
        let data = AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        let att = AggregatedAttestation {
            aggregation_bits: AggregationBits { bits: bits.clone() },
            data,
        };
        let stmt = statement_from_attestation(pool_profile_digest(), &att).unwrap();
        let proof = prove_type1(&stmt).unwrap();
        PoolEntry {
            proof,
            coverage: bits.iter().filter(|b| **b).count() as u32,
            inserted_slot: slot,
            attestation_ssz: att.ssz_encode(),
        }
    }

    #[test]
    fn merges_two_variants() {
        let a = entry(vec![true, false], 3);
        let b = entry(vec![false, true], 3);
        let root = AggregatedAttestation::ssz_decode(&a.attestation_ssz)
            .unwrap()
            .data
            .hash_tree_root();
        let key = PoolKey {
            profile_digest: pool_profile_digest(),
            message_root: root,
        };
        let mut pool = AggregatePool::new(4);
        pool.insert_verified(key, a);
        pool.insert_verified(key, b);
        match merge_best_pool_variants(&mut pool, key) {
            PoolMergeResult::Merged { coverage } => assert_eq!(coverage, 2),
            other => panic!("unexpected {other:?}"),
        }
    }
}
