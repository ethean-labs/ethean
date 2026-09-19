//! Insert attestation / aggregation gossip into the aggregate pool.

use crate::aggregation::{merge_best_pool_variants, AggregatePool, PoolEntry, PoolKey};
use crate::gossip_decode::try_decode_attestation;
use ethean_crypto::{domain_digest, PROD_AGGREGATION_FINGERPRINT};
use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, AggregationBits, MultiMessageAggregate, SignedAggregatedAttestation,
};

/// Profile digest shared with transition Type-2 verify.
pub fn pool_profile_digest() -> Hash32 {
    domain_digest(
        b"ethean-transition/v1/agg-profile",
        PROD_AGGREGATION_FINGERPRINT.as_bytes(),
    )
}

/// Decode and retain attestation or aggregation topic payloads.
///
/// Returns the pool message root when an entry was inserted.
pub fn ingest_into_pool(
    pool: &mut AggregatePool,
    topic: &str,
    payload: &[u8],
) -> Option<Hash32> {
    if topic.contains("/attestation_") {
        if let Ok(agg) = AggregatedAttestation::ssz_decode(payload) {
            let message_root = agg.data.hash_tree_root();
            let coverage = agg.aggregation_bits.bits.iter().filter(|b| **b).count() as u32;
            pool.insert_verified(
                PoolKey {
                    profile_digest: pool_profile_digest(),
                    message_root,
                },
                PoolEntry {
                    proof: Vec::new(),
                    coverage,
                    inserted_slot: agg.data.slot.get(),
                    attestation_ssz: agg.ssz_encode(),
                },
            );
            return Some(message_root);
        }
        if let Ok(signed) = SignedAggregatedAttestation::ssz_decode(payload) {
            let message_root = signed.data.hash_tree_root();
            let coverage = signed
                .proof
                .participants
                .bits
                .iter()
                .filter(|b| **b)
                .count() as u32;
            let reconstructed = AggregatedAttestation {
                aggregation_bits: AggregationBits {
                    bits: signed.proof.participants.bits.clone(),
                },
                data: signed.data.clone(),
            };
            pool.insert_verified(
                PoolKey {
                    profile_digest: pool_profile_digest(),
                    message_root,
                },
                PoolEntry {
                    proof: signed.proof.proof.clone(),
                    coverage,
                    inserted_slot: signed.data.slot.get(),
                    attestation_ssz: reconstructed.ssz_encode(),
                },
            );
            let key = PoolKey {
                profile_digest: pool_profile_digest(),
                message_root,
            };
            let _ = merge_best_pool_variants(pool, key);
            return Some(message_root);
        }
        // Fall through to generic decode path for content-only retention.
        if let Some(decoded) = try_decode_attestation(topic, payload) {
            pool.insert_verified(
                PoolKey {
                    profile_digest: pool_profile_digest(),
                    message_root: decoded.message_root,
                },
                PoolEntry {
                    proof: decoded.proof,
                    coverage: decoded.coverage,
                    inserted_slot: decoded.slot,
                    attestation_ssz: Vec::new(),
                },
            );
            return Some(decoded.message_root);
        }
    }
    if topic.contains("/aggregation/") {
        if let Ok(agg) = MultiMessageAggregate::new(payload.to_vec()) {
            if let Ok(root) = agg.hash_tree_root() {
                pool.insert_verified(
                    PoolKey {
                        profile_digest: pool_profile_digest(),
                        message_root: root,
                    },
                    PoolEntry {
                        proof: payload.to_vec(),
                        coverage: 0,
                        inserted_slot: 0,
                        attestation_ssz: Vec::new(),
                    },
                );
                return Some(root);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_types::{AttestationData, Checkpoint};

    #[test]
    fn attestation_gossip_lands_in_pool() {
        let mut pool = AggregatePool::new(4);
        let agg = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, false, true],
            },
            data: AttestationData {
                slot: Slot::new(5),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let enc = agg.ssz_encode();
        let topic = "/leanconsensus/abcd/attestation_1/ssz_snappy";
        let root = ingest_into_pool(&mut pool, topic, &enc).expect("insert");
        assert_eq!(root, agg.data.hash_tree_root());
        let best = pool
            .best(&PoolKey {
                profile_digest: pool_profile_digest(),
                message_root: root,
            })
            .expect("best");
        assert_eq!(best.coverage, 2);
        assert_eq!(best.inserted_slot, 5);
        assert_eq!(best.attestation_ssz, enc);
    }
}
