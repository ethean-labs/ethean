//! Proposer duty flow: refresh snapshot → build binding → reserve proposal key.

use crate::duty_gate::{evaluate_gate, DutyView, SuppressReason};
use crate::error::{Result, SignerError};
use crate::scheduler::DutyTick;
use crate::signer::{KeyId, Signer, SignerStore, SigningDuty, SigningRole, SigningRoot};
use ethean_crypto::{CryptoBackend, Signature};
use ethean_primitives::Hash32;

/// Inputs for one proposal attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposerPlan {
    /// Slot-boundary tick (interval 0 expected for publish).
    pub tick: DutyTick,
    /// Local proposal key.
    pub key_id: KeyId,
    /// Parent root selected from the chain snapshot.
    pub parent_root: Hash32,
    /// Block signing root after local transition (state root binding).
    pub block_signing_root: Hash32,
}

/// Outcome of a proposer attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposerOutcome {
    /// Proposal signature ready for Type-2 envelope assembly.
    Signed { signature: Signature, parent_root: Hash32 },
    /// Duty suppressed.
    Suppressed(SuppressReason),
    /// Build / sign failure.
    Failed(String),
}

/// Run proposer signing after gate and parent freshness checks.
pub fn run_proposer<B: CryptoBackend, S: SignerStore>(
    signer: &mut Signer<B, S>,
    view: &DutyView,
    plan: &ProposerPlan,
) -> Result<ProposerOutcome> {
    if let Err(reason) = evaluate_gate(view) {
        return Ok(ProposerOutcome::Suppressed(reason));
    }
    if plan.tick.slot.get() != view.wall_slot.get() {
        return Ok(ProposerOutcome::Suppressed(SuppressReason::HeadLagExceeded));
    }
    // Publish window is the slot boundary (interval 0); later intervals are late.
    if plan.tick.interval != 0 {
        return Ok(ProposerOutcome::Failed(
            "proposal publish after slot-boundary interval".into(),
        ));
    }
    let duty = SigningDuty {
        key_id: plan.key_id,
        role: SigningRole::Proposal,
        slot: plan.tick.slot.get() as u32,
        root: SigningRoot::from_bytes(plan.block_signing_root),
    };
    match signer.sign_duty(duty) {
        Ok(signature) => Ok(ProposerOutcome::Signed {
            signature,
            parent_root: plan.parent_root,
        }),
        Err(SignerError::ConflictingDuty) | Err(SignerError::LeafBurned) => {
            Err(SignerError::ConflictingDuty)
        }
        Err(e) => Ok(ProposerOutcome::Failed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::{record_from_keygen, InMemorySignerStore};
    use ethean_crypto::TestHmacBackend;
    use ethean_primitives::Slot;
    use std::sync::Arc;

    #[test]
    fn rejects_late_interval() {
        let backend = Arc::new(TestHmacBackend::new([2u8; 32]));
        let mut signer = Signer::new(backend.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            backend.as_ref(),
            KeyId::from_bytes([4u8; 16]),
            SigningRole::Proposal,
            0,
            64,
        )
        .unwrap();
        signer.import_key(rec).unwrap();
        let view = DutyView {
            wall_slot: Slot::new(3),
            genesis_slot: Slot::new(0),
            syncing: false,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: 2,
        };
        let plan = ProposerPlan {
            tick: DutyTick {
                slot: Slot::new(3),
                interval: 2,
                generation: 1,
            },
            key_id: KeyId::from_bytes([4u8; 16]),
            parent_root: [1u8; 32],
            block_signing_root: [2u8; 32],
        };
        match run_proposer(&mut signer, &view, &plan).unwrap() {
            ProposerOutcome::Failed(msg) => assert!(msg.contains("slot-boundary")),
            other => panic!("unexpected {other:?}"),
        }
    }
}
