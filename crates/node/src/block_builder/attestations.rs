//! Select proved aggregated attestations from the pool for block bodies.

use crate::aggregation::AggregatePool;
use ethean_types::{AggregatedAttestation, BlockBody};

/// Best-coverage proved entry per attestation data, in deterministic order,
/// with the Type-1 proof for each body attestation (parallel lists).
///
/// Only entries carrying a verified Type-1 proof are eligible: a block
/// proof must merge one proof per body attestation (leanSpec block building).
pub fn body_from_pool(pool: &AggregatePool, max_attestations: usize) -> (BlockBody, Vec<Vec<u8>>) {
    let mut attestations = Vec::new();
    let mut proofs = Vec::new();
    for (_key, entry) in pool.best_entries() {
        if attestations.len() >= max_attestations {
            break;
        }
        if entry.proof.is_empty() {
            continue;
        }
        let Ok(attestation) = AggregatedAttestation::ssz_decode(&entry.attestation_ssz) else {
            continue;
        };
        attestations.push(attestation);
        proofs.push(entry.proof);
    }
    match BlockBody::new(attestations) {
        Ok(body) => (body, proofs),
        Err(_) => (BlockBody::default(), Vec::new()),
    }
}
