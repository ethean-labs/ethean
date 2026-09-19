//! PROD_CONFIG constants pinned to Phase 00 / leanSpec.

/// Construction name matching Phase 00 lock.
pub const CONSTRUCTION: &str = "leanSpec_internal_xmss";

/// leanSig pin used for the production backend (when enabled).
pub const LEANSIG_REV: &str = "c08a3bae74b0d85379cab72dcbefa4091546ecbb";

/// Base-2 log of lifetime (2^32 epochs).
pub const LOG_LIFETIME: u32 = 32;

/// Number of hash chains (v).
pub const DIMENSION: u32 = 46;

/// Winternitz / encoding base.
pub const BASE: u32 = 8;

/// Digit extraction parameter Z.
pub const Z: u32 = 8;

/// Quotient Q with Q * BASE^Z == P - 1.
pub const Q: u32 = 127;

/// Target sum for aborting encoding.
pub const TARGET_SUM: u32 = 200;

/// Max encoding tries.
pub const MAX_TRIES: u32 = 100_000;

/// Public parameter length in field elements.
pub const PARAMETER_LENGTH: u32 = 5;

/// Tweak length in field elements.
pub const TWEAK_LENGTH_FIELD_ELEMENTS: u32 = 2;

/// Message length in field elements (Poseidon message hash input).
pub const MESSAGE_LENGTH_FIELD_ELEMENTS: u32 = 9;

/// Encoding randomness length in field elements.
pub const RAND_LENGTH_FIELD_ELEMENTS: u32 = 7;

/// Hash digest length in field elements.
pub const HASH_LENGTH_FIELD_ELEMENTS: u32 = 8;

/// Poseidon sponge capacity in field elements.
pub const CAPACITY: u32 = 9;

/// Wire public-key size (leanSpec Bytes52).
pub const PUBLIC_KEY_BYTES: usize = 52;

/// Wire signature size (leanSpec PROD_CONFIG SIGNATURE_LENGTH_BYTES).
pub const SIGNATURE_BYTES: usize = 2536;

/// Message length in bytes for the signature scheme.
pub const MESSAGE_BYTES: usize = 32;

/// Stable fingerprint string for startup asserts / lock files.
pub const PROD_FINGERPRINT: &str = concat!(
    "leanSpec_internal_xmss|LOG_LIFETIME=32|DIMENSION=46|BASE=8|Z=8|Q=127|",
    "TARGET_SUM=200|PK=52|SIG=2536|HASH_FE=8|PARAM_FE=5|RAND_FE=7|CAP=9"
);

/// Assert PROD wire sizes and core parameters (call at process start / tests).
pub fn assert_prod_invariants() {
    assert_eq!(PUBLIC_KEY_BYTES, 52);
    assert_eq!(SIGNATURE_BYTES, 2536);
    assert_eq!(DIMENSION, 46);
    assert_eq!(LOG_LIFETIME, 32);
    assert_eq!(BASE, 8);
    assert_eq!(TARGET_SUM, 200);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_invariants_hold() {
        assert_prod_invariants();
        assert!(PROD_FINGERPRINT.contains("PK=52"));
        assert!(PROD_FINGERPRINT.contains("SIG=2536"));
    }
}
