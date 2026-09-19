//! Crypto backend trait and implementations.

use crate::domain::DOMAIN_TEST_SCHEME;
use crate::error::{CryptoError, Result};
use crate::hash::domain_digest;
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::{MESSAGE_BYTES, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};

/// Opaque secret material handle (never Debug-printed as raw key bytes).
#[derive(Clone)]
pub struct SecretKeyMaterial {
    pub(crate) bytes: Vec<u8>,
    pub(crate) activation_epoch: u32,
    pub(crate) num_active_epochs: u32,
}

impl std::fmt::Debug for SecretKeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretKeyMaterial")
            .field("bytes", &"<redacted>")
            .field("activation_epoch", &self.activation_epoch)
            .field("num_active_epochs", &self.num_active_epochs)
            .finish()
    }
}

impl SecretKeyMaterial {
    /// Activation start epoch.
    pub fn activation_epoch(&self) -> u32 {
        self.activation_epoch
    }

    /// Number of active epochs.
    pub fn num_active_epochs(&self) -> u32 {
        self.num_active_epochs
    }
}

/// Backend for XMSS keygen / sign / verify.
pub trait CryptoBackend: Send + Sync {
    /// Human-readable backend name.
    fn name(&self) -> &'static str;

    /// Generate a key pair for `[activation, activation + num_active)`.
    fn key_gen(
        &self,
        activation_epoch: u32,
        num_active_epochs: u32,
    ) -> Result<(PublicKey, SecretKeyMaterial)>;

    /// Sign `message` at `epoch`.
    fn sign(
        &self,
        sk: &SecretKeyMaterial,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
    ) -> Result<Signature>;

    /// Verify signature; never silent-accept on error paths.
    fn verify(
        &self,
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool>;
}

/// Production backend: leanSig PROD, or fail-closed if the feature is off.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProductionBackend;

impl CryptoBackend for ProductionBackend {
    fn name(&self) -> &'static str {
        #[cfg(feature = "leansig-backend")]
        {
            "leansig-prod-aborting-l32-d46-b8"
        }
        #[cfg(not(feature = "leansig-backend"))]
        {
            "unavailable-leansig-not-compiled"
        }
    }

    fn key_gen(
        &self,
        activation_epoch: u32,
        num_active_epochs: u32,
    ) -> Result<(PublicKey, SecretKeyMaterial)> {
        #[cfg(feature = "leansig-backend")]
        {
            crate::backend_leansig::key_gen(activation_epoch, num_active_epochs)
        }
        #[cfg(not(feature = "leansig-backend"))]
        {
            let _ = (activation_epoch, num_active_epochs);
            Err(CryptoError::BackendUnavailable(
                "leansig-backend feature disabled; refuse fake production keys",
            ))
        }
    }

    fn sign(
        &self,
        sk: &SecretKeyMaterial,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
    ) -> Result<Signature> {
        #[cfg(feature = "leansig-backend")]
        {
            crate::backend_leansig::sign(sk, epoch, message)
        }
        #[cfg(not(feature = "leansig-backend"))]
        {
            let _ = (sk, epoch, message);
            Err(CryptoError::BackendUnavailable(
                "leansig-backend feature disabled; refuse fake production signatures",
            ))
        }
    }

    fn verify(
        &self,
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool> {
        #[cfg(feature = "leansig-backend")]
        {
            crate::backend_leansig::verify(pk, epoch, message, signature)
        }
        #[cfg(not(feature = "leansig-backend"))]
        {
            let _ = (pk, epoch, message, signature);
            Err(CryptoError::BackendUnavailable(
                "leansig-backend feature disabled; refuse always-true verify",
            ))
        }
    }
}

/// Deterministic HMAC-style backend with real verify (PROD wire sizes).
///
/// Enabled for unit tests and the `test-hmac` feature only.
#[cfg(any(test, feature = "test-hmac"))]
#[derive(Clone)]
pub struct TestHmacBackend {
    seed: [u8; 32],
}

#[cfg(any(test, feature = "test-hmac"))]
impl TestHmacBackend {
    /// Create from a 32-byte seed.
    pub fn new(seed: [u8; 32]) -> Self {
        Self { seed }
    }
}

#[cfg(any(test, feature = "test-hmac"))]
impl Default for TestHmacBackend {
    fn default() -> Self {
        Self::new([0x11; 32])
    }
}

#[cfg(any(test, feature = "test-hmac"))]
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
        let mut material = Vec::with_capacity(40);
        material.extend_from_slice(&self.seed);
        material.extend_from_slice(&activation_epoch.to_le_bytes());
        material.extend_from_slice(&num_active_epochs.to_le_bytes());
        let pk_digest = domain_digest(DOMAIN_TEST_SCHEME, &material);
        let mut pk_bytes = [0u8; PUBLIC_KEY_BYTES];
        pk_bytes[..32].copy_from_slice(&pk_digest);
        pk_bytes[32..36].copy_from_slice(&activation_epoch.to_le_bytes());
        pk_bytes[36..40].copy_from_slice(&num_active_epochs.to_le_bytes());
        let tag = domain_digest(b"ethean-crypto/v1/test-pk-pad", &pk_digest);
        pk_bytes[40..52].copy_from_slice(&tag[..12]);
        Ok((
            PublicKey::from_bytes(pk_bytes),
            SecretKeyMaterial {
                bytes: material,
                activation_epoch,
                num_active_epochs,
            },
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
        let act = u32::from_le_bytes(pk.as_bytes()[32..36].try_into().unwrap());
        let num = u32::from_le_bytes(pk.as_bytes()[36..40].try_into().unwrap());
        let mut sk_bytes = Vec::with_capacity(40);
        sk_bytes.extend_from_slice(&self.seed);
        sk_bytes.extend_from_slice(&act.to_le_bytes());
        sk_bytes.extend_from_slice(&num.to_le_bytes());
        let expected_pk = self.key_gen(act, num)?.0;
        if expected_pk.as_bytes() != pk.as_bytes() {
            return Ok(false);
        }
        let sk = SecretKeyMaterial {
            bytes: sk_bytes,
            activation_epoch: act,
            num_active_epochs: num,
        };
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
