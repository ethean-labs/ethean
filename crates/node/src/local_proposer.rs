//! Local proposer signer for duty-tick proposal binding.
//!
//! Default smoke path uses test-hmac. With `leansig-backend`, [`LocalProposer::try_production`]
//! attempts the real leanSig ProductionBackend (still fail-closed if vendor/FFI is broken).

use ethean_crypto::TestHmacBackend;
#[cfg(feature = "leansig-backend")]
use ethean_crypto::ProductionBackend;
use ethean_primitives::Hash32;
use ethean_validator::{
    record_from_keygen, run_proposer, DutyTick, DutyView, InMemorySignerStore, KeyId,
    ProposerOutcome, ProposerPlan, Signer, SigningRole,
};
use std::sync::Arc;

enum ProposerBackend {
    Hmac(Signer<TestHmacBackend, InMemorySignerStore>),
    #[cfg(feature = "leansig-backend")]
    LeanSig(Signer<ProductionBackend, InMemorySignerStore>),
}

/// In-process proposal key + durable journal.
pub struct LocalProposer {
    backend: ProposerBackend,
    key_id: KeyId,
    /// True when keys came from leanSig ProductionBackend.
    production: bool,
}

impl std::fmt::Debug for LocalProposer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalProposer")
            .field("key_id", &self.key_id)
            .field("production", &self.production)
            .finish_non_exhaustive()
    }
}

impl LocalProposer {
    /// Create a long-lived smoke proposal key on the test-hmac backend.
    pub fn smoke() -> Result<Self, String> {
        let crypto = Arc::new(TestHmacBackend::new([0xe7; 32]));
        let key_id = KeyId::from_bytes([0x50; 16]);
        let mut signer = Signer::new(crypto.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            crypto.as_ref(),
            key_id,
            SigningRole::Proposal,
            0,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        signer.import_key(rec).map_err(|e| e.to_string())?;
        Ok(Self {
            backend: ProposerBackend::Hmac(signer),
            key_id,
            production: false,
        })
    }

    /// Prefer leanSig production keys when the feature is on; otherwise smoke.
    pub fn prefer_production() -> Result<Self, String> {
        #[cfg(feature = "leansig-backend")]
        {
            match Self::try_production() {
                Ok(p) => return Ok(p),
                Err(e) => {
                    // Fall back so the node still signs in smoke/dev builds.
                    let _ = e;
                }
            }
        }
        Self::smoke()
    }

    /// Attempt leanSig ProductionBackend keygen (feature `leansig-backend` only).
    #[cfg(feature = "leansig-backend")]
    pub fn try_production() -> Result<Self, String> {
        let crypto = Arc::new(ProductionBackend);
        let key_id = KeyId::from_bytes([0x51; 16]);
        let mut signer = Signer::new(crypto.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            crypto.as_ref(),
            key_id,
            SigningRole::Proposal,
            0,
            64,
        )
        .map_err(|e| e.to_string())?;
        signer.import_key(rec).map_err(|e| e.to_string())?;
        Ok(Self {
            backend: ProposerBackend::LeanSig(signer),
            key_id,
            production: true,
        })
    }

    /// True when this instance uses leanSig production keys.
    pub fn is_production(&self) -> bool {
        self.production
    }

    /// Sign the planned block root via [`run_proposer`].
    pub fn sign_proposal(
        &mut self,
        tick: DutyTick,
        view: &DutyView,
        parent_root: Hash32,
        block_signing_root: Hash32,
    ) -> Result<Vec<u8>, String> {
        let plan = ProposerPlan {
            tick,
            key_id: self.key_id,
            parent_root,
            block_signing_root,
        };
        let outcome = match &mut self.backend {
            ProposerBackend::Hmac(signer) => {
                run_proposer(signer, view, &plan).map_err(|e| e.to_string())?
            }
            #[cfg(feature = "leansig-backend")]
            ProposerBackend::LeanSig(signer) => {
                run_proposer(signer, view, &plan).map_err(|e| e.to_string())?
            }
        };
        match outcome {
            ProposerOutcome::Signed { signature, .. } => Ok(signature.as_bytes().to_vec()),
            ProposerOutcome::Suppressed(reason) => Err(format!("suppressed:{reason:?}")),
            ProposerOutcome::Failed(msg) => Err(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;

    #[test]
    fn smoke_signs_slot_boundary() {
        let mut prop = LocalProposer::smoke().expect("smoke");
        assert!(!prop.is_production());
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        let view = DutyView {
            wall_slot: Slot::new(1),
            genesis_slot: Slot::new(0),
            syncing: false,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: 2,
        };
        let sig = prop
            .sign_proposal(tick, &view, [1u8; 32], [2u8; 32])
            .expect("sign");
        assert_eq!(sig.len(), ethean_crypto::SIGNATURE_BYTES);
    }

    #[test]
    fn prefer_production_falls_back_without_feature() {
        let prop = LocalProposer::prefer_production().expect("fallback");
        assert!(!prop.is_production());
    }
}
