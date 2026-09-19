//! Insert attestation / aggregation gossip into the aggregate pool.

use crate::aggregation::{AggregatePool, PoolEntry, PoolKey};
use crate::gossip_decode::try_decode_attestation;
use ethean_crypto::{domain_digest, PROD_AGGREGATION_FINGERPRINT};
use ethean_primitives::Hash32;
use ethean_types::MultiMessageAggregate;

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
            },
        );
        return Some(decoded.message_root);
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
    use ethean_types::{AggregatedAttestation, AggregationBits, AttestationData, Checkpoint};

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
    }
}
