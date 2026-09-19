//! Ethean Lean Consensus crypto — XMSS wire types and backends.
//!
//! Production verify uses pinned leanSig when `leansig-backend` is enabled.
//! Without that feature, production paths fail closed (never always-true).

#![forbid(unsafe_code)]

pub mod aggregate;
pub mod backend;
#[cfg(feature = "leansig-backend")]
mod backend_leansig;
pub mod domain;
pub mod error;
pub mod hash;
pub mod signature;
pub mod xmss;

pub use aggregate::AggregateProofDeferred;
pub use backend::{CryptoBackend, ProductionBackend, SecretKeyMaterial};
#[cfg(any(test, feature = "test-hmac"))]
pub use backend::TestHmacBackend;
pub use error::{CryptoError, Result};
pub use hash::{domain_digest, signature_hash, signing_root_digest, Digest32};
pub use signature::{verify, PublicKey, Signature};
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
        assert_eq!(PUBLIC_KEY_BYTES, 52);
        assert_eq!(SIGNATURE_BYTES, 2536);
        assert_eq!(DIMENSION, 46);
        assert_eq!(LOG_LIFETIME, 32);
    }
}
