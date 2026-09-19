//! Local proposer signer (test-hmac) for duty-tick proposal binding.

use ethean_crypto::TestHmacBackend;
use ethean_primitives::Hash32;
use ethean_validator::{
    record_from_keygen, run_proposer, DutyTick, DutyView, InMemorySignerStore, KeyId,
    ProposerOutcome, ProposerPlan, Signer, SigningRole,
};
use std::sync::Arc;

/// In-process proposal key + durable journal (smoke / test-hmac backend).
pub struct LocalProposer {
    signer: Signer<TestHmacBackend, InMemorySignerStore>,
    key_id: KeyId,
}

impl std::fmt::Debug for LocalProposer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalProposer")
            .field("key_id", &self.key_id)
            .finish_non_exhaustive()
    }
}

impl LocalProposer {
    /// Create a long-lived smoke proposal key on the test-hmac backend.
    pub fn smoke() -> Result<Self, String> {
        let backend = Arc::new(TestHmacBackend::new([0xe7; 32]));
        let key_id = KeyId::from_bytes([0x50; 16]);
        let mut signer = Signer::new(backend.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            backend.as_ref(),
            key_id,
            SigningRole::Proposal,
            0,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        signer.import_key(rec).map_err(|e| e.to_string())?;
        Ok(Self { signer, key_id })
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
        match run_proposer(&mut self.signer, view, &plan).map_err(|e| e.to_string())? {
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
}
