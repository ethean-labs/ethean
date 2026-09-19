//! High-level XMSS scheme helpers wired to a [`CryptoBackend`].

use crate::backend::{CryptoBackend, SecretKeyMaterial};
use crate::error::Result;
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::{assert_prod_invariants, MESSAGE_BYTES, PROD_FINGERPRINT};

/// Return the pinned PROD fingerprint string.
pub fn prod_fingerprint() -> &'static str {
    assert_prod_invariants();
    PROD_FINGERPRINT
}

/// Generate keys via `backend`.
pub fn key_gen(
    backend: &dyn CryptoBackend,
    activation_epoch: u32,
    num_active_epochs: u32,
) -> Result<(PublicKey, SecretKeyMaterial)> {
    backend.key_gen(activation_epoch, num_active_epochs)
}

/// Sign via `backend`.
pub fn sign(
    backend: &dyn CryptoBackend,
    sk: &SecretKeyMaterial,
    epoch: u32,
    message: &[u8; MESSAGE_BYTES],
) -> Result<Signature> {
    backend.sign(sk, epoch, message)
}

/// Verify via `backend` (bool; see also [`crate::signature::verify`]).
pub fn verify_bool(
    backend: &dyn CryptoBackend,
    pk: &PublicKey,
    epoch: u32,
    message: &[u8; MESSAGE_BYTES],
    signature: &Signature,
) -> Result<bool> {
    backend.verify(pk, epoch, message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::TestHmacBackend;

    #[test]
    fn fingerprint_stable() {
        assert!(prod_fingerprint().contains("DIMENSION=46"));
    }

    #[test]
    fn scheme_roundtrip() {
        let backend = TestHmacBackend::default();
        let (pk, sk) = key_gen(&backend, 0, 16).unwrap();
        let msg = [42u8; 32];
        let sig = sign(&backend, &sk, 3, &msg).unwrap();
        assert!(verify_bool(&backend, &pk, 3, &msg, &sig).unwrap());
    }
}
