//! Attester duty flow: snapshot → reserve → sign → local verify import path.

use crate::duty_gate::{evaluate_gate, DutyView, SuppressReason};
use crate::error::{Result, SignerError};
use crate::scheduler::DutyTick;
use crate::signer::{KeyId, Signer, SignerStore, SigningDuty, SigningRole, SigningRoot};
use ethean_crypto::{CryptoBackend, Signature};
use ethean_primitives::Hash32;

/// Inputs for one attestation attempt (immutable snapshot from chain owner).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttesterPlan {
    /// Scheduler tick that triggered this duty.
    pub tick: DutyTick,
    /// Local attestation key.
    pub key_id: KeyId,
    /// Head / target signing root for this slot.
    pub signing_root: Hash32,
    /// Assigned attestation subnet (opaque u16 until Phase 10).
    pub subnet: u16,
}

/// Outcome of an attester attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttesterOutcome {
    /// Signature produced and ready for local import / publish.
    Signed { signature: Signature, subnet: u16 },
    /// Duty suppressed by the gate.
    Suppressed(SuppressReason),
    /// Signing failed (conflict, burn, crypto).
    Failed(String),
}

/// Run attester: gate → reserve/sign via durable signer.
pub fn run_attester<B: CryptoBackend, S: SignerStore>(
    signer: &mut Signer<B, S>,
    view: &DutyView,
    plan: &AttesterPlan,
) -> Result<AttesterOutcome> {
    if let Err(reason) = evaluate_gate(view) {
        return Ok(AttesterOutcome::Suppressed(reason));
    }
    if plan.tick.slot.get() != view.wall_slot.get() {
        return Ok(AttesterOutcome::Suppressed(SuppressReason::HeadLagExceeded));
    }
    let duty = SigningDuty {
        key_id: plan.key_id,
        role: SigningRole::Attestation,
        slot: plan.tick.slot.get() as u32,
        root: SigningRoot::from_bytes(plan.signing_root),
    };
    match signer.sign_duty(duty) {
        Ok(signature) => Ok(AttesterOutcome::Signed {
            signature,
            subnet: plan.subnet,
        }),
        Err(SignerError::ConflictingDuty) | Err(SignerError::LeafBurned) => {
            Err(SignerError::ConflictingDuty)
        }
        Err(e) => Ok(AttesterOutcome::Failed(e.to_string())),
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
    fn suppresses_when_syncing() {
        let backend = Arc::new(TestHmacBackend::new([1u8; 32]));
        let mut signer = Signer::new(backend.clone(), InMemorySignerStore::default());
        let (rec, _) = record_from_keygen(
            backend.as_ref(),
            KeyId::from_bytes([9u8; 16]),
            SigningRole::Attestation,
            0,
            64,
        )
        .unwrap();
        signer.import_key(rec).unwrap();
        let view = DutyView {
            wall_slot: Slot::new(1),
            genesis_slot: Slot::new(0),
            syncing: true,
            parent_state_available: true,
            profile_matches: true,
            signer_safe: true,
            head_lag_slots: 0,
            max_head_lag_slots: 2,
        };
        let plan = AttesterPlan {
            tick: DutyTick {
                slot: Slot::new(1),
                interval: 1,
                generation: 1,
            },
            key_id: KeyId::from_bytes([9u8; 16]),
            signing_root: [7u8; 32],
            subnet: 0,
        };
        match run_attester(&mut signer, &view, &plan).unwrap() {
            AttesterOutcome::Suppressed(SuppressReason::Syncing) => {}
            other => panic!("unexpected {other:?}"),
        }
    }
}
