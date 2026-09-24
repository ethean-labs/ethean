//! Group proved pool entries by attestation data for block selection.

use crate::aggregation::AggregatePool;
use ethean_types::{AggregatedAttestation, AttestationData};

/// One verified Type-1 proof and the participant bits it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofVariant {
    pub bits: Vec<bool>,
    pub proof: Vec<u8>,
}

impl ProofVariant {
    /// Number of covered validators.
    pub fn coverage(&self) -> usize {
        self.bits.iter().filter(|b| **b).count()
    }
}

/// Every proved variant per attestation data (leanSpec `aggregated_payloads`).
///
/// Entries without a verified Type-1 proof are skipped: a block proof must
/// merge one proof per body attestation. Data order follows the pool's
/// message-root order; the selector re-sorts by target slot.
pub fn candidates_from_pool(pool: &AggregatePool) -> Vec<(AttestationData, Vec<ProofVariant>)> {
    let mut out: Vec<(AttestationData, Vec<ProofVariant>)> = Vec::new();
    for (key, _) in pool.best_entries() {
        for entry in pool.variants(&key).unwrap_or_default() {
            if entry.proof.is_empty() {
                continue;
            }
            let Ok(att) = AggregatedAttestation::ssz_decode(&entry.attestation_ssz) else {
                continue;
            };
            let variant = ProofVariant {
                bits: att.aggregation_bits.bits,
                proof: entry.proof,
            };
            match out.iter_mut().find(|(d, _)| *d == att.data) {
                Some((_, list)) => list.push(variant),
                None => out.push((att.data, vec![variant])),
            }
        }
    }
    out
}
