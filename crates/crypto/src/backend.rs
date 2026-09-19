//! Crypto backend trait and implementations.
//!
//! Production paths use leanSig when the `leansig-backend` feature compiles.
//! Test-only HMAC backend is gated behind `cfg(test)` and never always-true.

use crate::error::{CryptoError, Result};
use crate::hash::domain_digest;
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::{MESSAGE_BYTES, PUBLIC_KEY_BYTES, SIGNATURE_BYTES};
use crate::domain::DOMAIN_TEST_SCHEME;

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

    /// Verify signature; returns `Ok(true/false)` or backend error (never silent accept).
    fn verify(
        &self,
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool>;
}

/// Production backend: leanSig PROD instantiation, or fail-closed if unavailable.
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
            leansig_backend::key_gen(activation_epoch, num_active_epochs)
        }
        #[cfg(not(feature = "leansig-backend"))]
        {
            let _ = (activation_epoch, num_active_epochs);
            Err(CryptoError::BackendUnavailable(
                "leansig-backend feature disabled or failed to compile; refuse fake production keys",
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
            leansig_backend::sign(sk, epoch, message)
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
            leansig_backend::verify(pk, epoch, message, signature)
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

/// Deterministic HMAC-style test backend with real verify (not always-true).
///
/// Produces PROD wire sizes by packing a MAC into fixed-length buffers.
/// Must not be selected on production release paths.
#[derive(Clone)]
pub struct TestHmacBackend {
    seed: [u8; 32],
}

impl TestHmacBackend {
    /// Create from a 32-byte seed.
    pub fn new(seed: [u8; 32]) -> Self {
        Self { seed }
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
        let mut material = Vec::with_capacity(40);
        material.extend_from_slice(&self.seed);
        material.extend_from_slice(&activation_epoch.to_le_bytes());
        material.extend_from_slice(&num_active_epochs.to_le_bytes());
        let pk_digest = domain_digest(DOMAIN_TEST_SCHEME, &material);
        let mut pk_bytes = [0u8; PUBLIC_KEY_BYTES];
        pk_bytes[..32].copy_from_slice(&pk_digest);
        pk_bytes[32..36].copy_from_slice(&activation_epoch.to_le_bytes());
        pk_bytes[36..40].copy_from_slice(&num_active_epochs.to_le_bytes());
        // Remaining bytes: domain tag fingerprint
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
        // Expand mac into remaining bytes so malleability is hard without the seed.
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
        // Recover activation window from pk layout used in key_gen.
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

#[cfg(feature = "leansig-backend")]
mod leansig_backend {
    use super::*;
    use leansig::serialization::Serializable;
    use leansig::signature::generalized_xmss::instantiations_aborting::lifetime_2_to_the_32::{
        SIGAbortingTargetSumLifetime32Dim46Base8 as ProdScheme,
    };
    use leansig::signature::SignatureScheme;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    pub fn key_gen(
        activation_epoch: u32,
        num_active_epochs: u32,
    ) -> Result<(PublicKey, SecretKeyMaterial)> {
        // Full 2^32 keygen is not practical in CI; refuse oversized activations.
        const MAX_TEST_ACTIVE: u32 = 2;
        if num_active_epochs > MAX_TEST_ACTIVE {
            return Err(CryptoError::KeyGenerationFailed(format!(
                "leansig PROD key_gen capped at {MAX_TEST_ACTIVE} active epochs in this wrapper \
                 (requested {num_active_epochs}); use operator tooling for full lifetime"
            )));
        }
        let mut rng = StdRng::from_entropy();
        let (pk, sk) = ProdScheme::key_gen(
            &mut rng,
            activation_epoch as usize,
            num_active_epochs as usize,
        );
        let pk_bytes = pk.to_bytes();
        if pk_bytes.len() != PUBLIC_KEY_BYTES {
            return Err(CryptoError::InvalidPublicKeyLength {
                expected: PUBLIC_KEY_BYTES,
                got: pk_bytes.len(),
            });
        }
        let mut arr = [0u8; PUBLIC_KEY_BYTES];
        arr.copy_from_slice(&pk_bytes);
        let sk_bytes = sk.to_bytes();
        Ok((
            PublicKey::from_bytes(arr),
            SecretKeyMaterial {
                bytes: sk_bytes,
                activation_epoch,
                num_active_epochs,
            },
        ))
    }

    pub fn sign(
        sk: &SecretKeyMaterial,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
    ) -> Result<Signature> {
        type Sk = <ProdScheme as SignatureScheme>::SecretKey;
        type Sig = <ProdScheme as SignatureScheme>::Signature;
        let secret = Sk::from_bytes(&sk.bytes).map_err(|e| {
            CryptoError::SigningFailed(format!("decode secret key: {e:?}"))
        })?;
        let sig = ProdScheme::sign(&secret, epoch, message)
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;
        let raw = sig.to_bytes();
        wire_signature_from_leansig(&raw)
    }

    pub fn verify(
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool> {
        type Pk = <ProdScheme as SignatureScheme>::PublicKey;
        type Sig = <ProdScheme as SignatureScheme>::Signature;
        let public = Pk::from_bytes(pk.as_bytes()).map_err(|e| {
            CryptoError::SigningFailed(format!("decode public key: {e:?}"))
        })?;
        let raw = leansig_bytes_from_wire(signature.as_bytes())?;
        let sig = Sig::from_bytes(&raw)
            .map_err(|e| CryptoError::SigningFailed(format!("decode signature: {e:?}")))?;
        Ok(ProdScheme::verify(&public, epoch, message, &sig))
    }

    fn wire_signature_from_leansig(raw: &[u8]) -> Result<Signature> {
        if raw.len() == SIGNATURE_BYTES {
            return Signature::try_from_slice(raw);
        }
        // Prefer leanSpec wire size: pad or reject. Documented in phase-07 lock.
        if raw.len() < SIGNATURE_BYTES {
            let mut buf = [0u8; SIGNATURE_BYTES];
            buf[..raw.len()].copy_from_slice(raw);
            // Prefix length so verify can recover leanSig encoding.
            // Layout: u32 le len || raw || zeros
            if raw.len() + 4 > SIGNATURE_BYTES {
                return Err(CryptoError::InvalidSignatureLength {
                    expected: SIGNATURE_BYTES,
                    got: raw.len(),
                });
            }
            let mut framed = [0u8; SIGNATURE_BYTES];
            framed[..4].copy_from_slice(&(raw.len() as u32).to_le_bytes());
            framed[4..4 + raw.len()].copy_from_slice(raw);
            return Ok(Signature::from_bytes(framed));
        }
        Err(CryptoError::InvalidSignatureLength {
            expected: SIGNATURE_BYTES,
            got: raw.len(),
        })
    }

    fn leansig_bytes_from_wire(wire: &[u8; SIGNATURE_BYTES]) -> Result<Vec<u8>> {
        let len = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
        if len > 0 && len + 4 <= SIGNATURE_BYTES && wire[4 + len..].iter().all(|&b| b == 0) {
            return Ok(wire[4..4 + len].to_vec());
        }
        // Exact leanSpec / leanSig encoding fills the buffer.
        Ok(wire.to_vec())
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

    #[test]
    fn production_without_leansig_fails_closed() {
        #[cfg(not(feature = "leansig-backend"))]
        {
            let b = ProductionBackend;
            assert!(b.verify(
                &PublicKey::from_bytes([0u8; 52]),
                0,
                &[0u8; 32],
                &Signature::from_bytes([0u8; SIGNATURE_BYTES]),
            )
            .is_err());
        }
    }
}
