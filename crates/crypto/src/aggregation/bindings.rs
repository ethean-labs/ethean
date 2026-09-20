//! Pinned aggregation / leanVM parameters.

/// leanVM commit pinned by Phase 08 plan.
pub const LEANVM_REV: &str = "e2592df4e30fdddbbf8ae26a333116c68cec7026";

/// Inverse-rate exponent for production SNARK backend (leanSpec PROD).
///
/// pq-devnet-4 high-level plan (`leanEthereum/pm`) allows protocol values in
/// `1..=4`; Ethean's pinned generation uses `2` (leanBench default).
pub const LOG_INV_RATE: u32 = 2;

/// Maximum proof payload bytes (`ByteList512KiB` / Phase 08 bound).
pub const MAX_PROOF_BYTES: usize = 524_288;

/// Maximum Type-2 components: proposer + up to MAX_ATTESTATIONS_DATA attestation proofs.
pub const MAX_TYPE2_COMPONENTS: usize = 1 + 8;

/// Stable fingerprint for startup asserts and phase locks.
pub const PROD_AGGREGATION_FINGERPRINT: &str = concat!(
    "leanVM|rev=e2592df4e30fdddbbf8ae26a333116c68cec7026|",
    "LOG_INV_RATE=2|MAX_PROOF=524288|TYPE2_COMP_MAX=9|gen=type1_type2"
);

/// Return the aggregation fingerprint after asserting invariants.
pub fn aggregation_fingerprint() -> &'static str {
    assert_aggregation_invariants();
    PROD_AGGREGATION_FINGERPRINT
}

/// Assert proof size and rate constants.
pub fn assert_aggregation_invariants() {
    assert_eq!(MAX_PROOF_BYTES, 512 * 1024);
    assert_eq!(LOG_INV_RATE, 2);
    assert!((1..=4).contains(&LOG_INV_RATE), "LOG_INV_RATE outside D4 1..=4");
    assert_eq!(MAX_TYPE2_COMPONENTS, 9);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invariants_hold() {
        assert_aggregation_invariants();
        assert!(aggregation_fingerprint().contains("LOG_INV_RATE=2"));
    }
}
