//! Select aggregated attestations from the gossip pool for block bodies.

use crate::aggregation::AggregatePool;
use ethean_types::{AggregatedAttestation, BlockBody};

/// Build a `BlockBody` from the highest-coverage pool entries (deterministic order).
///
/// Prefers entries that already carry a Type-1 proof (D3 Type-1 cache), then fills with
/// SSZ-only rows up to `max_attestations`. Skips entries without retained attestation SSZ.
pub fn body_from_pool(pool: &AggregatePool, max_attestations: usize) -> BlockBody {
    let mut proved = Vec::new();
    let mut plain = Vec::new();
    for (_key, entry) in pool.best_entries() {
        if entry.attestation_ssz.is_empty() {
            continue;
        }
        let Ok(agg) = AggregatedAttestation::ssz_decode(&entry.attestation_ssz) else {
            continue;
        };
        if !entry.proof.is_empty() {
            proved.push(agg);
        } else {
            plain.push(agg);
        }
    }
    let mut attestations = Vec::with_capacity(max_attestations);
    for agg in proved.into_iter().chain(plain) {
        if attestations.len() >= max_attestations {
            break;
        }
        attestations.push(agg);
    }
    BlockBody::new(attestations).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::{PoolEntry, PoolKey};
    use ethean_primitives::Slot;
    use ethean_types::{AggregationBits, AttestationData, Checkpoint};

    #[test]
    fn packs_retained_attestations_in_root_order() {
        let mut pool = AggregatePool::new(4);
        let low = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true, false],
            },
            data: AttestationData {
                slot: Slot::new(1),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let high = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true],
            },
            data: AttestationData {
                slot: Slot::new(2),
                head: Checkpoint {
                    root: [0xff; 32],
                    slot: Slot::new(0),
                },
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let root_low = low.data.hash_tree_root();
        let root_high = high.data.hash_tree_root();
        assert!(root_low < root_high);
        pool.insert_verified(
            PoolKey {
                profile_digest: [0u8; 32],
                message_root: root_high,
            },
            PoolEntry {
                proof: Vec::new(),
                coverage: 1,
                inserted_slot: 2,
                attestation_ssz: high.ssz_encode(),
            },
        );
        pool.insert_verified(
            PoolKey {
                profile_digest: [0u8; 32],
                message_root: root_low,
            },
            PoolEntry {
                proof: Vec::new(),
                coverage: 2,
                inserted_slot: 1,
                attestation_ssz: low.ssz_encode(),
            },
        );
        let body = body_from_pool(&pool, 8);
        assert_eq!(body.attestations.len(), 2);
        assert_eq!(body.attestations[0].data.hash_tree_root(), root_low);
        assert_eq!(body.attestations[1].data.hash_tree_root(), root_high);
    }

    #[test]
    fn prefers_type1_proved_entries_first() {
        let mut pool = AggregatePool::new(4);
        let plain = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true],
            },
            data: AttestationData {
                slot: Slot::new(1),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let proved = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true],
            },
            data: AttestationData {
                slot: Slot::new(2),
                head: Checkpoint {
                    root: [0xaa; 32],
                    slot: Slot::new(0),
                },
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        // Insert plain first (lower root likely) but with no proof.
        pool.insert_verified(
            PoolKey {
                profile_digest: [0u8; 32],
                message_root: plain.data.hash_tree_root(),
            },
            PoolEntry {
                proof: Vec::new(),
                coverage: 9,
                inserted_slot: 1,
                attestation_ssz: plain.ssz_encode(),
            },
        );
        pool.insert_verified(
            PoolKey {
                profile_digest: [0u8; 32],
                message_root: proved.data.hash_tree_root(),
            },
            PoolEntry {
                proof: vec![1, 2, 3],
                coverage: 1,
                inserted_slot: 2,
                attestation_ssz: proved.ssz_encode(),
            },
        );
        let body = body_from_pool(&pool, 1);
        assert_eq!(body.attestations.len(), 1);
        assert_eq!(
            body.attestations[0].data.hash_tree_root(),
            proved.data.hash_tree_root()
        );
    }
}
