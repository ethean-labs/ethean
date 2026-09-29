//! Collect finished proofs from the proof service (each wall step and
//! between duty intervals).

use crate::block_builder::PlanTransition;
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::proof_service::ProofOutcome;
use std::time::Duration;

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
                elapsed,
            } => {
                let result = proof.and_then(|p| {
                    crate::aggregation_duty::accept_attestation_proof(
                        owner,
                        data,
                        participants,
                        p,
                        elapsed,
                    )
                });
                match result {
                    Ok(event) => events.push(event),
                    Err(error) => events.push(failed("attestation", error)),
                }
            }
            ProofOutcome::Split {
                data,
                participants,
                proof,
                elapsed,
            } => {
                let result = proof.and_then(|p| {
                    crate::aggregation_duty::accept_recovered_proof(
                        owner,
                        data,
                        participants,
                        p,
                        elapsed,
                    )
                });
                match result {
                    Ok(event) => events.push(event),
                    Err(error) => events.push(failed("split", error)),
                }
            }
            ProofOutcome::Block {
                plan,
                proof,
                elapsed,
            } => match proof {
                Ok(proof) if slot_not_reached(owner, &plan) => {
                    tracing::debug!(
                        slot = plan.block.slot.get(),
                        "block proof ready before its slot; holding until the boundary"
                    );
                    owner.deferred_block_proof = Some(DeferredBlockProof {
                        plan,
                        proof,
                        elapsed,
                    });
                }
                Ok(proof) => events.extend(accept_block(owner, plan, proof, elapsed)),
                Err(error) => {
                    owner.block_proof_slot = None;
                    events.push(failed("block", error));
                }
            },
        }
    }
    events
}

/// A block proved ahead of its slot (built during the previous slot's last
/// interval). Import and gossip wait for the wall clock.
#[derive(Debug, Clone)]
pub struct DeferredBlockProof {
    /// Plan the proof was built for.
    pub plan: PlanTransition,
    /// Merged Type-2 proof bytes.
    pub proof: Vec<u8>,
    /// Prover wall time.
    pub elapsed: Duration,
}

/// Import a held block proof once the wall clock has reached its slot.
pub fn release_deferred_block(owner: &mut ChainOwner) -> Vec<ChainEvent> {
    let ready = owner
        .deferred_block_proof
        .as_ref()
        .is_some_and(|d| !slot_not_reached(owner, &d.plan));
    if !ready {
        return Vec::new();
    }
    let Some(held) = owner.deferred_block_proof.take() else {
        return Vec::new();
    };
    accept_block(owner, held.plan, held.proof, held.elapsed)
}

fn slot_not_reached(owner: &ChainOwner, plan: &PlanTransition) -> bool {
    owner
        .last_tick
        .is_some_and(|t| t.slot.get() < plan.block.slot.get())
}

fn accept_block(
    owner: &mut ChainOwner,
    plan: PlanTransition,
    proof: Vec<u8>,
    elapsed: Duration,
) -> Vec<ChainEvent> {
    match crate::duty_propose::accept_block_proof(owner, plan, proof, elapsed) {
        Ok(events) => events,
        Err(error) => {
            owner.block_proof_slot = None;
            vec![failed("block", error)]
        }
    }
}

fn failed(kind: &'static str, error: String) -> ChainEvent {
    tracing::warn!(kind, %error, "proof job failed");
    ChainEvent::ProofFailed { kind, error }
}
