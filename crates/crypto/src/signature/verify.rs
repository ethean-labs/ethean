//! Verification entry points (fail closed; never always-true on production paths).

use crate::error::{CryptoError, Result};
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::MESSAGE_BYTES;
use crate::backend::CryptoBackend;

/// Verify a message under `pk` at `epoch` using the active backend.
pub fn verify(
    backend: &dyn CryptoBackend,
    pk: &PublicKey,
    epoch: u32,
    message: &[u8; MESSAGE_BYTES],
    signature: &Signature,
) -> Result<()> {
    if backend.verify(pk, epoch, message, signature)? {
        Ok(())
    } else {
        Err(CryptoError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::TestHmacBackend;
    use crate::{key_gen, sign};

    #[test]
    fn test_backend_roundtrip() {
        let backend = TestHmacBackend::new([7u8; 32]);
        let (pk, sk) = key_gen(&backend, 0, 8).unwrap();
        let msg = [3u8; 32];
        let sig = sign(&backend, &sk, 0, &msg).unwrap();
        verify(&backend, &pk, 0, &msg, &sig).unwrap();
        assert!(verify(&backend, &pk, 0, &[4u8; 32], &sig).is_err());
    }
}
