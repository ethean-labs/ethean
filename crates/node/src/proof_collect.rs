//! Collect finished proofs from the proof service at the start of each step.

use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::proof_service::ProofOutcome;

/// Drain finished jobs, verify and apply them. Failures are reported, never fatal.
pub fn collect_proofs(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let Some(prover) = owner.prover.as_mut() else {
        return Vec::new();
    };
    let outcomes = prover.drain();
    let mut events = Vec::new();
    for outcome in outcomes {
        match outcome {
            ProofOutcome::Attestation {
                data,
                participants,
                proof,
            } => {
                let result = proof.and_then(|p| {
                    crate::aggregation_duty::accept_attestation_proof(owner, data, participants, p)
                });
                match result {
                    Ok(event) => events.push(event),
                    Err(error) => events.push(failed("attestation", error)),
                }
            }
            ProofOutcome::Block { plan, proof } => {
                let result =
                    proof.and_then(|p| crate::duty_propose::accept_block_proof(owner, plan, p));
                match result {
                    Ok(mut evs) => events.append(&mut evs),
                    Err(error) => events.push(failed("block", error)),
                }
            }
        }
    }
    events
}

fn failed(kind: &'static str, error: String) -> ChainEvent {
    tracing::warn!(kind, %error, "proof job failed");
    ChainEvent::ProofFailed { kind, error }
}
