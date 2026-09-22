//! Deterministic HMAC-style test backend with PROD wire sizes.
//!
//! Real verify (never always-true), but no post-quantum security: unit tests
//! and local smoke only.

use crate::backend::{CryptoBackend, SecretKeyMaterial};
use crate::domain::DOMAIN_TEST_SCHEME;
use crate::error::{CryptoError, Result};
use crate::hash::domain_digest;
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::{MESSAGE_BYTES, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};

/// Deterministic HMAC-style backend with real verify (PROD wire sizes).
///
/// Enabled for unit tests and the `test-hmac` feature only.
#[derive(Clone)]
pub struct TestHmacBackend {
    seed: [u8; 32],
}

impl TestHmacBackend {
    /// Create from a 32-byte seed.
    pub fn new(seed: [u8; 32]) -> Self {
        Self { seed }
    }

    fn material(&self, activation_epoch: u32, num_active_epochs: u32) -> Vec<u8> {
        let mut material = Vec::with_capacity(40);
        material.extend_from_slice(&self.seed);
        material.extend_from_slice(&activation_epoch.to_le_bytes());
        material.extend_from_slice(&num_active_epochs.to_le_bytes());
        material
    }
}

impl Default for TestHmacBackend {
    fn default() -> Self {
        Self::new([0x11; 32])
    }
}

impl CryptoBackend for TestHmacBackend {
    fn name(&self) -> &'static str {
        "test-hmac-fixed-wire"
    }

    fn key_gen(
        &self,
        activation_epoch: u32,
        num_active_epochs: u32,
    ) -> Result<(PublicKey, SecretKeyMaterial)> {
        if num_active_epochs == 0 {
            return Err(CryptoError::KeyGenerationFailed(
                "num_active_epochs must be > 0".into(),
            ));
        }
        let material = self.material(activation_epoch, num_active_epochs);
        let pk_digest = domain_digest(DOMAIN_TEST_SCHEME, &material);
        let mut pk_bytes = [0u8; PUBLIC_KEY_BYTES];
        pk_bytes[..32].copy_from_slice(&pk_digest);
        pk_bytes[32..36].copy_from_slice(&activation_epoch.to_le_bytes());
        pk_bytes[36..40].copy_from_slice(&num_active_epochs.to_le_bytes());
        let tag = domain_digest(b"ethean-crypto/v1/test-pk-pad", &pk_digest);
        pk_bytes[40..52].copy_from_slice(&tag[..12]);
        Ok((
            PublicKey::from_bytes(pk_bytes),
            SecretKeyMaterial::from_imported(material, activation_epoch, num_active_epochs),
        ))
    }

    fn sign(
        &self,
        sk: &SecretKeyMaterial,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
    ) -> Result<Signature> {
        let start = sk.activation_epoch;
        let end = start.saturating_add(sk.num_active_epochs);
        if epoch < start || epoch >= end {
            return Err(CryptoError::LifetimeExhausted);
        }
        let mut payload = Vec::with_capacity(sk.bytes.len() + 4 + MESSAGE_BYTES);
        payload.extend_from_slice(&sk.bytes);
        payload.extend_from_slice(&epoch.to_le_bytes());
        payload.extend_from_slice(message);
        let mac = domain_digest(DOMAIN_TEST_SCHEME, &payload);
        let mut bytes = [0u8; SIGNATURE_BYTES];
        bytes[..32].copy_from_slice(&mac);
        bytes[32..36].copy_from_slice(&epoch.to_le_bytes());
        bytes[36..68].copy_from_slice(message);
        let mut block = mac;
        let mut offset = 68;
        while offset < SIGNATURE_BYTES {
            block = domain_digest(b"ethean-crypto/v1/test-sig-expand", &block);
            let take = (SIGNATURE_BYTES - offset).min(32);
            bytes[offset..offset + take].copy_from_slice(&block[..take]);
            offset += take;
        }
        Ok(Signature::from_bytes(bytes))
    }

    fn verify(
        &self,
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool> {
        let act = u32::from_le_bytes(pk.as_bytes()[32..36].try_into().expect("4 bytes"));
        let num = u32::from_le_bytes(pk.as_bytes()[36..40].try_into().expect("4 bytes"));
        let expected_pk = self.key_gen(act, num)?.0;
        if expected_pk.as_bytes() != pk.as_bytes() {
            return Ok(false);
        }
        let sk = SecretKeyMaterial::from_imported(self.material(act, num), act, num);
        let expected = self.sign(&sk, epoch, message)?;
        Ok(expected.as_bytes() == signature.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_rejects_wrong_message() {
        let b = TestHmacBackend::new([9u8; 32]);
        let (pk, sk) = b.key_gen(0, 4).unwrap();
        let msg = [1u8; 32];
        let sig = b.sign(&sk, 1, &msg).unwrap();
        assert!(b.verify(&pk, 1, &msg, &sig).unwrap());
        assert!(!b.verify(&pk, 1, &[2u8; 32], &sig).unwrap());
    }
}
