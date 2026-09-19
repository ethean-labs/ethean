//! Durable reservation journal (in-memory with explicit flush).

use crate::error::{Result, SignerError};
use crate::signer::duty::{KeyId, ReservationStatus, SigningDuty};
use crate::signer::keystore::KeyRecord;
use ethean_crypto::{Digest32, Signature};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Pending {
    duty: SigningDuty,
    durable: bool,
}

#[derive(Clone)]
struct Completed {
    duty: SigningDuty,
    signature: Signature,
    #[allow(dead_code)]
    signature_hash: Digest32,
}

/// Persistence contract for signer state (Phase 11 can plug RocksDB).
pub trait SignerStore: Send {
    fn put_key(&mut self, record: KeyRecord) -> Result<()>;
    fn get_key(&self, key_id: &KeyId) -> Result<Option<KeyRecord>>;
    fn reserve(&mut self, duty: &SigningDuty) -> Result<ReservationStatus>;
    fn flush(&mut self) -> Result<()>;
    fn is_reserved_durable(&self, duty: &SigningDuty) -> Result<bool>;
    fn complete(&mut self, duty: &SigningDuty, sig: Signature, hash: Digest32) -> Result<()>;
    fn completed_signature(&self, duty: &SigningDuty) -> Result<Option<Signature>>;
    fn is_burned(&self, duty: &SigningDuty) -> Result<bool>;
    fn burn_uncertain(&mut self, duty: &SigningDuty) -> Result<()>;
    fn highest_reserved_leaf(&self, key_id: &KeyId) -> Result<Option<u32>>;
}

/// In-memory store with an explicit flush barrier for crash tests.
#[derive(Default)]
pub struct InMemorySignerStore {
    keys: HashMap<KeyId, KeyRecord>,
    /// Unflushed reservations.
    dirty: HashMap<(KeyId, u32, crate::signer::duty::SigningRole), Pending>,
    /// Flushed reservations awaiting completion.
    reserved: HashMap<(KeyId, u32, crate::signer::duty::SigningRole), Pending>,
    completed: HashMap<(KeyId, u32, crate::signer::duty::SigningRole), Completed>,
    burned: HashSet<(KeyId, u32, crate::signer::duty::SigningRole)>,
    /// Simulate crash: next flush fails once.
    pub fail_next_flush: bool,
}

impl InMemorySignerStore {
    fn slot_key(
        duty: &SigningDuty,
    ) -> (KeyId, u32, crate::signer::duty::SigningRole) {
        (duty.key_id, duty.slot, duty.role)
    }
}

impl SignerStore for InMemorySignerStore {
    fn put_key(&mut self, record: KeyRecord) -> Result<()> {
        self.keys.insert(record.key_id, record);
        Ok(())
    }

    fn get_key(&self, key_id: &KeyId) -> Result<Option<KeyRecord>> {
        Ok(self.keys.get(key_id).cloned())
    }

    fn reserve(&mut self, duty: &SigningDuty) -> Result<ReservationStatus> {
        let k = Self::slot_key(duty);
        if self.burned.contains(&k) {
            return Err(SignerError::LeafBurned);
        }
        if let Some(done) = self.completed.get(&k) {
            if done.duty.root == duty.root {
                return Ok(ReservationStatus::AlreadyCompleted);
            }
            return Ok(ReservationStatus::Conflict);
        }
        if let Some(p) = self.reserved.get(&k).or_else(|| self.dirty.get(&k)) {
            if p.duty.root != duty.root {
                return Ok(ReservationStatus::Conflict);
            }
            return Ok(if p.durable {
                ReservationStatus::Reserved
            } else {
                ReservationStatus::PendingFlush
            });
        }
        self.dirty.insert(
            k,
            Pending {
                duty: duty.clone(),
                durable: false,
            },
        );
        Ok(ReservationStatus::PendingFlush)
    }

    fn flush(&mut self) -> Result<()> {
        if self.fail_next_flush {
            self.fail_next_flush = false;
            return Err(SignerError::Store("simulated flush failure".into()));
        }
        let pending: Vec<_> = self.dirty.drain().collect();
        for (k, mut p) in pending {
            p.durable = true;
            self.reserved.insert(k, p);
        }
        Ok(())
    }

    fn is_reserved_durable(&self, duty: &SigningDuty) -> Result<bool> {
        let k = Self::slot_key(duty);
        Ok(self
            .reserved
            .get(&k)
            .map(|p| p.durable && p.duty.root == duty.root)
            .unwrap_or(false))
    }

    fn complete(&mut self, duty: &SigningDuty, sig: Signature, hash: Digest32) -> Result<()> {
        let k = Self::slot_key(duty);
        self.reserved.remove(&k);
        self.dirty.remove(&k);
        self.completed.insert(
            k,
            Completed {
                duty: duty.clone(),
                signature: sig,
                signature_hash: hash,
            },
        );
        Ok(())
    }

    fn completed_signature(&self, duty: &SigningDuty) -> Result<Option<Signature>> {
        let k = Self::slot_key(duty);
        Ok(self.completed.get(&k).and_then(|c| {
            if c.duty.root == duty.root {
                Some(c.signature.clone())
            } else {
                None
            }
        }))
    }

    fn is_burned(&self, duty: &SigningDuty) -> Result<bool> {
        Ok(self.burned.contains(&Self::slot_key(duty)))
    }

    fn burn_uncertain(&mut self, duty: &SigningDuty) -> Result<()> {
        let k = Self::slot_key(duty);
        self.dirty.remove(&k);
        self.reserved.remove(&k);
        self.burned.insert(k);
        Ok(())
    }

    fn highest_reserved_leaf(&self, key_id: &KeyId) -> Result<Option<u32>> {
        let mut max = None;
        for (kid, slot, _) in self
            .reserved
            .keys()
            .chain(self.completed.keys())
            .chain(self.dirty.keys())
        {
            if kid == key_id {
                max = Some(max.map_or(*slot, |m: u32| m.max(*slot)));
            }
        }
        Ok(max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::duty::{SigningRole, SigningRoot};

    #[test]
    fn flush_failure_keeps_reservation_non_durable() {
        let mut store = InMemorySignerStore::default();
        let duty = SigningDuty {
            key_id: KeyId::from_bytes([0u8; 16]),
            role: SigningRole::Attestation,
            slot: 1,
            root: SigningRoot::from_bytes([1u8; 32]),
        };
        store.fail_next_flush = true;
        assert!(matches!(
            store.reserve(&duty).unwrap(),
            ReservationStatus::PendingFlush
        ));
        assert!(store.flush().is_err());
        assert!(!store.is_reserved_durable(&duty).unwrap());
    }
}
