//! Crypto backend trait and implementations.
//!
//! [`ProductionBackend`] is the dependency-free leanSpec XMSS implementation
//! in [`crate::xmss::native`]. [`TestHmacBackend`] (see `backend_test_hmac`) is a
//! fast stand-in with PROD wire sizes for unit tests only.

use std::sync::{Arc, Mutex, OnceLock};

use crate::error::{CryptoError, Result};
use crate::signature::{PublicKey, Signature};
use crate::xmss::config::MESSAGE_BYTES;
#[cfg(test)]
use crate::xmss::config::{PUBLIC_KEY_BYTES, SIGNATURE_BYTES};

#[cfg(any(test, feature = "test-hmac"))]
pub use crate::backend_test_hmac::TestHmacBackend;
use crate::xmss::native::{self, OsRandom, XmssPublicKey, XmssSecretKey, XmssSignature, PROD};

type SharedXmss = Arc<Mutex<XmssSecretKey>>;

/// Opaque secret material handle (never Debug-printed as raw key bytes).
///
/// For XMSS keys the SSZ bytes are decoded once, on first use, and the
/// decoded key (with its sliding preparation window) is shared by clones.
#[derive(Clone)]
pub struct SecretKeyMaterial {
    pub(crate) bytes: Vec<u8>,
    pub(crate) activation_epoch: u32,
    pub(crate) num_active_epochs: u32,
    xmss: OnceLock<std::result::Result<SharedXmss, String>>,
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
    /// Import opaque secret bytes (Hive / lean-quickstart `*.ssz` privkey files).
    /// XMSS decoding is deferred to the first production sign.
    pub fn from_imported(bytes: Vec<u8>, activation_epoch: u32, num_active_epochs: u32) -> Self {
        Self {
            bytes,
            activation_epoch,
            num_active_epochs,
            xmss: OnceLock::new(),
        }
    }

    /// Decode a leanSpec XMSS secret key now; the activation window is taken
    /// from the key itself.
    pub fn from_xmss_ssz(bytes: Vec<u8>) -> Result<Self> {
        let sk = XmssSecretKey::from_ssz(&bytes)?;
        Ok(Self::from_xmss_with_bytes(sk, bytes))
    }

    /// Wrap an in-memory XMSS secret key.
    pub fn from_xmss(sk: XmssSecretKey) -> Self {
        let bytes = sk.to_ssz();
        Self::from_xmss_with_bytes(sk, bytes)
    }

    fn from_xmss_with_bytes(sk: XmssSecretKey, bytes: Vec<u8>) -> Self {
        let interval = sk.activation_interval();
        let material = Self {
            bytes,
            activation_epoch: interval.start.min(u32::MAX as u64) as u32,
            num_active_epochs: (interval.end - interval.start).min(u32::MAX as u64) as u32,
            xmss: OnceLock::new(),
        };
        let _ = material.xmss.set(Ok(Arc::new(Mutex::new(sk))));
        material
    }

    /// Shared decoded XMSS key (decoding on first call).
    pub fn xmss(&self) -> Result<SharedXmss> {
        self.xmss
            .get_or_init(|| {
                XmssSecretKey::from_ssz(&self.bytes)
                    .map(|sk| Arc::new(Mutex::new(sk)))
                    .map_err(|e| e.to_string())
            })
            .clone()
            .map_err(CryptoError::SigningFailed)
    }

    /// Public key derived from the XMSS secret key.
    pub fn xmss_public_key(&self) -> Result<PublicKey> {
        let shared = self.xmss()?;
        let guard = shared
            .lock()
            .map_err(|_| CryptoError::SigningFailed("key lock".into()))?;
        Ok(PublicKey::from_bytes(guard.public_key().to_ssz()))
    }

    /// Build the bottom trees needed to sign at `epoch` (blocking; call from a
    /// background task ahead of time).
    pub fn prepare_for_epoch(&self, epoch: u32) -> Result<()> {
        let shared = self.xmss()?;
        let mut guard = shared
            .lock()
            .map_err(|_| CryptoError::SigningFailed("key lock".into()))?;
        native::prepare_for_epoch(&PROD, &mut guard, epoch as u64)
    }

    /// Activation start epoch.
    pub fn activation_epoch(&self) -> u32 {
        self.activation_epoch
    }

    /// Number of active epochs.
    pub fn num_active_epochs(&self) -> u32 {
        self.num_active_epochs
    }

    /// Borrow raw secret bytes (callers must not log this).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
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

/// Production backend: native leanSpec PROD XMSS (lifetime `2^32`, 46 chains).
#[derive(Debug, Default, Clone, Copy)]
pub struct ProductionBackend;

impl ProductionBackend {
    /// Decode a wire signature for the PROD scheme.
    pub fn decode_signature(signature: &Signature) -> Result<XmssSignature> {
        XmssSignature::from_ssz(&PROD, signature.as_bytes())
    }

    /// Decode a wire public key.
    pub fn decode_public_key(pk: &PublicKey) -> Result<XmssPublicKey> {
        XmssPublicKey::from_ssz(pk.as_bytes())
    }
}

impl CryptoBackend for ProductionBackend {
    fn name(&self) -> &'static str {
        "ethean-native-xmss-prod-l32-d46-b8"
    }

    fn key_gen(
        &self,
        activation_epoch: u32,
        num_active_epochs: u32,
    ) -> Result<(PublicKey, SecretKeyMaterial)> {
        let (pk, sk) = native::key_gen(
            &PROD,
            &mut OsRandom,
            activation_epoch as u64,
            num_active_epochs as u64,
        )?;
        Ok((
            PublicKey::from_bytes(pk.to_ssz()),
            SecretKeyMaterial::from_xmss(sk),
        ))
    }

    fn sign(
        &self,
        sk: &SecretKeyMaterial,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
    ) -> Result<Signature> {
        let shared = sk.xmss()?;
        let mut guard = shared
            .lock()
            .map_err(|_| CryptoError::SigningFailed("key lock".into()))?;
        native::prepare_for_epoch(&PROD, &mut guard, epoch as u64)?;
        let sig = native::sign(&PROD, &guard, epoch, message)?;
        Signature::try_from_slice(&sig.to_ssz())
    }

    fn verify(
        &self,
        pk: &PublicKey,
        epoch: u32,
        message: &[u8; MESSAGE_BYTES],
        signature: &Signature,
    ) -> Result<bool> {
        let (Ok(pk), Ok(sig)) = (
            Self::decode_public_key(pk),
            Self::decode_signature(signature),
        ) else {
            return Ok(false);
        };
        Ok(native::verify(&PROD, &pk, epoch, message, &sig))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_verify_rejects_garbage_without_error() {
        let pk = PublicKey::from_bytes([0u8; PUBLIC_KEY_BYTES]);
        let sig = Signature::from_bytes([0u8; SIGNATURE_BYTES]);
        assert!(!ProductionBackend.verify(&pk, 0, &[0u8; 32], &sig).unwrap());
        let mut bad_pk = [0u8; PUBLIC_KEY_BYTES];
        bad_pk[..4].copy_from_slice(&crate::field::P.to_le_bytes());
        let pk = PublicKey::from_bytes(bad_pk);
        assert!(!ProductionBackend.verify(&pk, 0, &[0u8; 32], &sig).unwrap());
    }
}
