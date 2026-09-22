//! Local attester signer for duty-tick attestation binding.
//!
//! Mirrors [`crate::local_proposer::LocalProposer`]: smoke HMAC by default;
//! Hive registry keys install via [`LocalAttester::from_key_record`] on the
//! native XMSS backend.

use ethean_crypto::ProductionBackend;
use ethean_crypto::TestHmacBackend;
use ethean_primitives::Hash32;
use ethean_validator::{
    record_from_keygen, run_attester, AttesterOutcome, AttesterPlan, DutyTick, DutyView,
    InMemorySignerStore, KeyId, KeyRecord, Signer, SigningDuty, SigningRole, SigningRoot,
};
use std::sync::Arc;

enum AttesterBackend {
    Hmac(Signer<TestHmacBackend, InMemorySignerStore>),
    LeanSig(Signer<ProductionBackend, InMemorySignerStore>),
}

/// In-process attestation key + durable journal.
pub struct LocalAttester {
    backend: AttesterBackend,
    key_id: KeyId,
    /// True when keys are native XMSS (generated or imported).
    production: bool,
}

impl std::fmt::Debug for LocalAttester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalAttester")
            .field("key_id", &self.key_id)
            .field("production", &self.production)
            .finish_non_exhaustive()
    }
}

impl LocalAttester {
    /// Create a long-lived smoke attestation key on the test-hmac backend.
    pub fn smoke() -> Result<Self, String> {
        let crypto = Arc::new(TestHmacBackend::new([0xa7; 32]));
        let key_id = KeyId::from_bytes([0xa1; 16]);
        let mut signer = Signer::new(crypto.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            crypto.as_ref(),
            key_id,
            SigningRole::Attestation,
            0,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        signer.import_key(rec).map_err(|e| e.to_string())?;
        Ok(Self {
            backend: AttesterBackend::Hmac(signer),
            key_id,
            production: false,
        })
    }

    /// Import a pre-loaded attestation [`KeyRecord`] (Hive registry privkey).
    pub fn from_key_record(record: KeyRecord) -> Result<Self, String> {
        if record.role != SigningRole::Attestation {
            return Err(format!(
                "expected Attestation key record, got {:?}",
                record.role
            ));
        }
        let crypto = Arc::new(ProductionBackend);
        let key_id = record.key_id;
        let mut signer = Signer::new(crypto, InMemorySignerStore::default());
        signer.import_key(record).map_err(|e| e.to_string())?;
        Ok(Self {
            backend: AttesterBackend::LeanSig(signer),
            key_id,
            production: true,
        })
    }

    /// True when this instance uses native XMSS keys.
    pub fn is_production(&self) -> bool {
        self.production
    }

    /// Sign an attestation-data tree root via [`run_attester`].
    pub fn sign_attestation(
        &mut self,
        tick: DutyTick,
        view: &DutyView,
        signing_root: Hash32,
        subnet: u16,
    ) -> Result<Vec<u8>, String> {
        let plan = AttesterPlan {
            tick,
            key_id: self.key_id,
            signing_root,
            subnet,
        };
        let outcome = match &mut self.backend {
            AttesterBackend::Hmac(signer) => {
                run_attester(signer, view, &plan).map_err(|e| e.to_string())?
            }
            AttesterBackend::LeanSig(signer) => {
                run_attester(signer, view, &plan).map_err(|e| e.to_string())?
            }
        };
        match outcome {
            AttesterOutcome::Signed { signature, .. } => Ok(signature.as_bytes().to_vec()),
            AttesterOutcome::Suppressed(reason) => Err(format!("suppressed:{reason:?}")),
            AttesterOutcome::Failed(msg) => Err(msg),
        }
    }

    /// Re-verify an attestation signature against the local key.
    pub fn verify_attestation(
        &self,
        tick: DutyTick,
        signing_root: Hash32,
        signature: &[u8],
    ) -> Result<(), String> {
        let sig = ethean_crypto::Signature::try_from_slice(signature).map_err(|e| e.to_string())?;
        let duty = SigningDuty {
            key_id: self.key_id,
            role: SigningRole::Attestation,
            slot: tick.slot.get() as u32,
            root: SigningRoot::from_bytes(signing_root),
        };
        match &self.backend {
            AttesterBackend::Hmac(signer) => {
                signer.verify_duty(&duty, &sig).map_err(|e| e.to_string())
            }
            AttesterBackend::LeanSig(signer) => {
                signer.verify_duty(&duty, &sig).map_err(|e| e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;

    #[test]
    fn smoke_signs_attestation_root() {
        let mut att = LocalAttester::smoke().expect("smoke");
        assert!(!att.is_production());
        let tick = DutyTick {
            slot: Slot::new(2),
            interval: 1,
            generation: 1,
        };
        let view = DutyView {
            wall_slot: Slot::new(2),
            genesis_slot: Slot::new(0),
            syncing: false,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: 2,
        };
        let sig = att
            .sign_attestation(tick, &view, [9u8; 32], 0)
            .expect("sign");
        assert_eq!(sig.len(), ethean_crypto::SIGNATURE_BYTES);
        att.verify_attestation(tick, [9u8; 32], &sig)
            .expect("verify");
    }
}
