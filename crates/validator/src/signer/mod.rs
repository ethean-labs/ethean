//! Role-separated durable XMSS signer.

mod duty;
mod journal;
mod keystore;
mod recovery;

pub use duty::{KeyId, ReservationStatus, SigningDuty, SigningRole, SigningRoot};
pub use journal::{InMemorySignerStore, SignerStore};
pub use keystore::KeyRecord;
pub use recovery::reconcile_on_startup;

use crate::error::{Result, SignerError};
use ethean_crypto::{
    signature_hash, sign, verify, CryptoBackend, PublicKey, SecretKeyMaterial, Signature,
};
use std::sync::Arc;

/// Owner of keys + durable journal. Signs only after a flushed reservation.
pub struct Signer<B: CryptoBackend, S: SignerStore> {
    backend: Arc<B>,
    store: S,
}

impl<B: CryptoBackend, S: SignerStore> Signer<B, S> {
    /// Construct a signer bound to one backend and store.
    pub fn new(backend: Arc<B>, store: S) -> Self {
        Self { backend, store }
    }

    /// Import a key record (attestation or proposal). Secrets stay redacted.
    pub fn import_key(&mut self, record: KeyRecord) -> Result<()> {
        self.store.put_key(record)
    }

    /// Sign a duty: reserve → flush → crypto sign → complete.
    /// Exact retries return the prior signature.
    pub fn sign_duty(&mut self, duty: SigningDuty) -> Result<Signature> {
        if let Some(completed) = self.store.completed_signature(&duty)? {
            return Ok(completed);
        }
        if self.store.is_burned(&duty)? {
            return Err(SignerError::LeafBurned);
        }

        let record = self.store.get_key(&duty.key_id)?.ok_or(SignerError::KeyNotFound)?;
        if record.role != duty.role {
            return Err(SignerError::CrossRoleKeyUse);
        }
        if duty.slot < record.activation_slot
            || duty.slot >= record.activation_slot.saturating_add(record.num_active_slots)
        {
            return Err(SignerError::LifetimeExhausted);
        }

        match self.store.reserve(&duty)? {
            ReservationStatus::AlreadyCompleted => {
                return self
                    .store
                    .completed_signature(&duty)?
                    .ok_or_else(|| SignerError::Store("missing completed signature".into()));
            }
            ReservationStatus::Conflict => return Err(SignerError::ConflictingDuty),
            ReservationStatus::Reserved | ReservationStatus::PendingFlush => {}
        }

        self.store.flush()?;
        if !self.store.is_reserved_durable(&duty)? {
            return Err(SignerError::ReservationNotDurable);
        }

        let sig = sign(
            self.backend.as_ref(),
            &record.secret,
            duty.slot,
            duty.root.as_bytes(),
        )
        .map_err(|e| SignerError::Crypto(e.to_string()))?;

        verify(
            self.backend.as_ref(),
            &record.public_key,
            duty.slot,
            duty.root.as_bytes(),
            &sig,
        )
        .map_err(|e| SignerError::Crypto(e.to_string()))?;

        let hash = signature_hash(sig.as_bytes());
        self.store.complete(&duty, sig.clone(), hash)?;
        self.store.flush()?;
        Ok(sig)
    }

    /// Borrow the store (tests / recovery).
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Mutable store access.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    /// Public key for a key id, if present.
    pub fn public_key(&self, key_id: &KeyId) -> Result<Option<PublicKey>> {
        Ok(self.store.get_key(key_id)?.map(|k| k.public_key))
    }
}

/// Helper to build a key record from backend keygen.
pub fn record_from_keygen<B: CryptoBackend>(
    backend: &B,
    key_id: KeyId,
    role: SigningRole,
    activation_slot: u32,
    num_active_slots: u32,
) -> Result<(KeyRecord, SecretKeyMaterial)> {
    let (pk, sk) = ethean_crypto::key_gen(backend, activation_slot, num_active_slots)
        .map_err(|e| SignerError::Crypto(e.to_string()))?;
    Ok((
        KeyRecord {
            key_id,
            role,
            public_key: pk,
            secret: sk.clone(),
            activation_slot,
            num_active_slots,
            journal_generation: 1,
        },
        sk,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::TestHmacBackend;

    fn setup() -> Signer<TestHmacBackend, InMemorySignerStore> {
        let backend = Arc::new(TestHmacBackend::new([3u8; 32]));
        let mut signer = Signer::new(backend.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            backend.as_ref(),
            KeyId::from_bytes([1u8; 16]),
            SigningRole::Attestation,
            0,
            64,
        )
        .unwrap();
        signer.import_key(rec).unwrap();
        signer
    }

    #[test]
    fn idempotent_retry() {
        let mut s = setup();
        let duty = SigningDuty {
            key_id: KeyId::from_bytes([1u8; 16]),
            role: SigningRole::Attestation,
            slot: 2,
            root: SigningRoot::from_bytes([9u8; 32]),
        };
        let a = s.sign_duty(duty.clone()).unwrap();
        let b = s.sign_duty(duty).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn conflicting_root_rejected() {
        let mut s = setup();
        let d1 = SigningDuty {
            key_id: KeyId::from_bytes([1u8; 16]),
            role: SigningRole::Attestation,
            slot: 4,
            root: SigningRoot::from_bytes([1u8; 32]),
        };
        s.sign_duty(d1).unwrap();
        let d2 = SigningDuty {
            key_id: KeyId::from_bytes([1u8; 16]),
            role: SigningRole::Attestation,
            slot: 4,
            root: SigningRoot::from_bytes([2u8; 32]),
        };
        assert_eq!(s.sign_duty(d2).unwrap_err(), SignerError::ConflictingDuty);
    }

    #[test]
    fn cross_role_rejected() {
        let mut s = setup();
        let duty = SigningDuty {
            key_id: KeyId::from_bytes([1u8; 16]),
            role: SigningRole::Proposal,
            slot: 1,
            root: SigningRoot::from_bytes([7u8; 32]),
        };
        assert_eq!(s.sign_duty(duty).unwrap_err(), SignerError::CrossRoleKeyUse);
    }
}
