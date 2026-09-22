//! Ethean Lean Consensus crypto — XMSS wire types, aggregation, backends.
//!
//! XMSS is implemented natively (see [`xmss::native`]) and verified against
//! leanSpec vectors and published leanSig keys. Aggregate proofs are verified
//! through [`AggregateVerifier`], implemented by `ethean-multisig`.

#![forbid(unsafe_code)]

pub mod aggregation;
pub mod backend;
#[cfg(any(test, feature = "test-hmac"))]
mod backend_test_hmac;
pub mod batch;
pub mod domain;
pub mod error;
pub mod field;
pub mod hash;
pub mod keccak;
pub mod poseidon;
pub mod signature;
pub mod xmss;

pub use aggregation::{
    aggregation_fingerprint, assert_aggregation_invariants, AggregateVerifier, ProofComponent,
    LEANVM_REV, LOG_INV_RATE, MAX_PROOF_BYTES, PROD_AGGREGATION_FINGERPRINT,
};
#[cfg(any(test, feature = "test-hmac"))]
pub use backend::TestHmacBackend;
pub use backend::{CryptoBackend, ProductionBackend, SecretKeyMaterial};
pub use batch::{verify_batch, BatchVerifyItem, PublicKeyCache};
pub use error::{CryptoError, Result};
pub use field::Fp;
pub use hash::{domain_digest, signature_hash, signing_root_digest, Digest32};
pub use signature::{verify, PublicKey, Signature};
pub use xmss::native::{
    OsRandom, RandomSource, SchemeParams, SeededRandom, XmssPublicKey, XmssSecretKey,
    XmssSignature, PROD as XMSS_PROD, TEST as XMSS_TEST,
};
pub use xmss::{
    assert_prod_invariants, key_gen, prod_fingerprint, sign, verify_bool, BASE, CONSTRUCTION,
    DIMENSION, LEANSIG_REV, LOG_LIFETIME, MESSAGE_BYTES, PROD_FINGERPRINT, PUBLIC_KEY_BYTES,
    SIGNATURE_BYTES, TARGET_SUM,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_invariants() {
        assert_prod_invariants();
        assert_aggregation_invariants();
        assert_eq!(PUBLIC_KEY_BYTES, 52);
        assert_eq!(SIGNATURE_BYTES, 2536);
        assert_eq!(MAX_PROOF_BYTES, 524_288);
        assert_eq!(LOG_INV_RATE, 2);
    }
}
