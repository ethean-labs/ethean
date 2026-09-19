//! Node crypto surface — Lean XMSS via `ethean-crypto` (Phase 07).
//!
//! Legacy BLS / local WOTS modules are removed from production paths.

pub use ethean_crypto::{
    assert_prod_invariants, domain_digest, key_gen, prod_fingerprint, sign, signature_hash,
    signing_root_digest, verify, verify_bool, CryptoBackend, CryptoError, Digest32,
    ProductionBackend, PublicKey, SecretKeyMaterial, Signature, MESSAGE_BYTES, PUBLIC_KEY_BYTES,
    SIGNATURE_BYTES,
};

#[cfg(any(test, feature = "test-hmac"))]
pub use ethean_crypto::TestHmacBackend;

/// Crypto result type (node-local alias).
pub type Result<T> = std::result::Result<T, CryptoError>;
