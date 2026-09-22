//! Proposer duty: plan a block from proved pool entries, sign its root with
//! the proposal key, and hand the Type-2 merge to the proof service. The
//! block is published once the merged proof returns and verifies
//! ([`accept_block_proof`]).

use crate::block_builder::{
    assemble_signed_block, decide_publish, encode_proposal_gossip, plan_from_pool, PlanTransition,
    PublishDecision,
};
use crate::chain_owner::ChainOwner;
use crate::duty_propose_gate::is_assigned_proposer;
use crate::events::ChainEvent;
use crate::proof_service::ProofJob;
use crate::registry_keys_view::{attestation_keys_for_bits, proposal_key};
use ethean_crypto::Signature;
use ethean_multisig::{KeyedProof, LeanMultisigVerifier};
use ethean_primitives::ValidatorIndex;
use ethean_transition::{apply_block, TransitionContext};
use ethean_validator::DutyTick;

/// Plan a proposal for an owned slot and request its block proof.
pub fn try_plan_proposal(owner: &mut ChainOwner, tick: DutyTick) -> Vec<ChainEvent> {
    let mut out = Vec::new();
    let min_slot = tick.slot.get().saturating_sub(owner.max_head_lag_slots);
    owner.aggregates.prune_before(min_slot);
    owner.signatures.prune_before(min_slot);

    let (pre, profile) = match (owner.head_state.clone(), owner.profile.clone()) {
        (Some(s), Some(p)) => (s, p),
        _ => return out,
    };
    let n = pre.validators.len() as u64;
    if n == 0 || !is_assigned_proposer(owner, tick.slot.get(), n) {
        return out;
    }
    let proposer = ValidatorIndex::new(tick.slot.get() % n);
    let mut plan = match plan_from_pool(
        &owner.aggregates,
        owner.head_root,
        tick.slot,
        proposer,
        &pre,
        profile,
        16,
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(error = %e, slot = tick.slot.get(), "proposal plan skipped");
            return out;
        }
    };

    if owner.local_finality {
        crate::local_finality::inject_local_aggregate(owner, &mut plan);
        if let Err(e) = crate::local_finality::rebind_plan_state_root(owner, &mut plan) {
            tracing::debug!(error = %e, "local attestation rebind skipped");
        }
    }

    // Interval 0 is the publish window. Local finality also accepts the first
    // tick of a future slot so solo runs survive a skipped interval.
    let future_slot = tick.slot.get() > pre.slot.get();
    let publish_allowed = matches!(decide_publish(tick, tick, true), PublishDecision::Allow)
        && future_slot
        && (tick.interval == 0 || owner.local_finality);
    let Ok(root) = plan.block_root() else {
        return out;
    };
    owner.planned_proposal = Some(plan.clone());
    owner.planned_tick = Some(tick);
    out.push(ChainEvent::ProposalPlanned {
        root,
        slot: tick.slot.get(),
        attestations: plan.block.body.attestations.len(),
        publish_allowed,
    });
    if !publish_allowed {
        return out;
    }
    if owner.local_finality {
        // Smoke mode: injected votes carry no signatures, so the block cannot be
        // proved. It advances the local head only and is never gossiped.
        apply_locally(owner, &plan, tick);
        return out;
    }
    match request_block_proof(owner, tick, plan) {
        Ok(events) => out.extend(events),
        Err(reason) => {
            tracing::info!(slot = tick.slot.get(), %reason, "block proof not requested");
        }
    }
    out
}

fn request_block_proof(
    owner: &mut ChainOwner,
    tick: DutyTick,
    plan: PlanTransition,
) -> Result<Vec<ChainEvent>, String> {
    if owner
        .prover
        .as_ref()
        .ok_or("no prover available")?
        .block_in_flight()
    {
        return Err("a block proof is already in flight".into());
    }
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let proposer_key = proposal_key(state, plan.block.proposer_index.get())?;
    let mut attestation_proofs = Vec::with_capacity(plan.attestation_proofs.len());
    for (attestation, proof) in plan
        .block
        .body
        .attestations
        .iter()
        .zip(&plan.attestation_proofs)
    {
        attestation_proofs.push(KeyedProof {
            public_keys: attestation_keys_for_bits(state, &attestation.aggregation_bits.bits)?,
            proof: proof.clone(),
        });
    }
    let root = plan.block_root()?;
    let duty_view = owner.snapshot(tick.slot, 0).duty_view;
    let local = owner.proposer.as_mut().ok_or("no local proposer")?;
    if !local.is_production() {
        return Err("proposer key is a smoke key, not an XMSS registry key".into());
    }
    let raw = local.sign_proposal(tick, &duty_view, plan.parent_root, root)?;
    local.verify_proposal(tick, root, &raw)?;
    let proposer_signature = Signature::try_from_slice(&raw).map_err(|e| e.to_string())?;
    let job = ProofJob::Block {
        plan,
        attestation_proofs,
        proposer_key,
        proposer_signature,
    };
    if !owner.prover.as_mut().is_some_and(|p| p.submit(job)) {
        return Err("prover queue is full".into());
    }
    Ok(vec![
        ChainEvent::ProposalSigned {
            root,
            signature_len: raw.len(),
        },
        ChainEvent::ProofScheduled {
            kind: "block",
            root,
        },
    ])
}

/// Verify a returned block proof against the parent registry, import the
/// block, and queue it for gossip. Stale plans (head moved) are dropped.
pub fn accept_block_proof(
    owner: &mut ChainOwner,
    mut plan: PlanTransition,
    proof: Vec<u8>,
) -> Result<Vec<ChainEvent>, String> {
    if owner.head_root != plan.parent_root {
        return Err("head moved while the block proof was built".into());
    }
    let pre = owner.head_state.clone().ok_or("no head state")?;
    let profile = owner.profile.clone().ok_or("no profile")?;
    plan.aggregate_proof = proof;
    let signed = assemble_signed_block(&plan)?;
    let ctx = TransitionContext::new(profile.clone());
    let applied = apply_block(&pre, &signed, &ctx, &LeanMultisigVerifier)
        .map_err(|e| format!("own block rejected: {e}"))?;
    let gossip = encode_proposal_gossip(&plan, profile.fork_name)?;
    let root = gossip.block_root;
    owner.head_state = Some(applied.post_state);
    owner.advance_head(root, plan.parent_root);
    owner.remember_durable_block(root, gossip.payload.clone());
    let events = vec![
        ChainEvent::BlockProofAttached {
            root,
            proof_len: gossip.proof_len,
        },
        ChainEvent::ProposalGossipReady {
            root,
            topic: gossip.topic.clone(),
            payload_len: gossip.payload.len(),
            proof_len: gossip.proof_len,
        },
        ChainEvent::HeadUpdated {
            root,
            slot: plan.block.slot.get(),
        },
    ];
    owner.pending_block_gossip = Some(gossip);
    Ok(events)
}

fn apply_locally(owner: &mut ChainOwner, plan: &PlanTransition, tick: DutyTick) {
    match crate::local_finality::apply_planned_locally(owner, plan) {
        Ok(applied) => {
            crate::local_finality::promote_local_checkpoints(owner, applied);
            tracing::info!(
                slot = tick.slot.get(),
                finalized = owner
                    .head_state
                    .as_ref()
                    .map(|s| s.latest_finalized.slot.get())
                    .unwrap_or(0),
                justified = owner
                    .head_state
                    .as_ref()
                    .map(|s| s.latest_justified.slot.get())
                    .unwrap_or(0),
                "local finality applied proposal to head"
            );
        }
        Err(e) => tracing::warn!(error = %e, "local finality apply failed"),
    }
}

#[cfg(test)]
#[path = "duty_propose_tests.rs"]
mod tests;
