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
use ethean_metrics::lean::{inc, observe_since};
use ethean_multisig::{KeyedProof, LeanMultisigVerifier};
use ethean_primitives::{Slot, ValidatorIndex};
use ethean_transition::{apply_block, TransitionContext};
use ethean_types::{MultiMessageAggregate, SignedBlock};
use ethean_validator::DutyTick;
use std::time::Instant;

/// Plan a proposal for an owned slot and request its block proof.
pub fn try_plan_proposal(owner: &mut ChainOwner, tick: DutyTick) -> Vec<ChainEvent> {
    let mut out = Vec::new();
    owner.prune_untrusted_pools(tick.slot.get());

    let Some(profile) = owner.profile.clone() else {
        return out;
    };
    let Some(pre) = owner.head_state.as_ref() else {
        return out;
    };
    let n = pre.validators.len() as u64;
    let pre_slot = pre.slot.get();
    if n == 0 || !is_assigned_proposer(owner, tick.slot.get(), n) {
        return out;
    }
    // The head already holds a block for this slot (ours or imported); later
    // ticks of the same slot have nothing to build.
    if pre_slot >= tick.slot.get() {
        return out;
    }
    // Already proving (or holding) this slot's block, e.g. one started during
    // the previous slot's last interval. Re-planning would sign the slot twice.
    if owner.block_proof_slot == Some(tick.slot.get()) {
        return out;
    }
    // One block proof at a time: planning now would be thrown away.
    if !owner.local_finality && owner.prover.as_ref().is_some_and(|p| p.block_in_flight()) {
        tracing::debug!(
            slot = tick.slot.get(),
            "proposal waits: an earlier block proof is still running"
        );
        return out;
    }
    let proposer = ValidatorIndex::new(tick.slot.get() % n);
    let build_started = Instant::now();
    let known_roots = owner.known_block_roots();
    // Solo blocks carry no proof, so only proved blocks pay the merge budget.
    let max_data = if owner.local_finality {
        ethean_types::MAX_ATTESTATIONS_DATA
    } else {
        owner.block_attestation_data_cap()
    };
    let planned = match owner.head_state.as_ref() {
        Some(pre) => plan_from_pool(
            &owner.aggregates,
            owner.head_root,
            tick.slot,
            proposer,
            pre,
            profile,
            &known_roots,
            max_data,
        ),
        None => return out,
    };
    observe_since("lean_block_building_time_seconds", &[], build_started);
    let mut plan = match planned {
        Ok(p) => {
            inc("lean_block_building_success_total", &[], 1.0);
            p
        }
        Err(e) => {
            inc("lean_block_building_failures_total", &[], 1.0);
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

    // Interval 0 is the preferred publish window. Intervals 1–2 still allow a
    // Type-2 request when interval 0 was blocked by a busy prover queue
    // (same-client meshes). Local finality accepts any first tick of a future slot.
    let future_slot = tick.slot.get() > pre_slot;
    let publish_allowed = matches!(decide_publish(tick, tick, true), PublishDecision::Allow)
        && future_slot
        && (tick.interval <= 2 || owner.local_finality);
    let Ok(root) = plan.block_root() else {
        return out;
    };
    owner.planned_tick = Some(tick);
    out.push(ChainEvent::ProposalPlanned {
        root,
        slot: tick.slot.get(),
        attestations: plan.block.body.attestations.len(),
        publish_allowed,
    });
    if !publish_allowed {
        owner.planned_proposal = Some(plan);
        return out;
    }
    if owner.local_finality {
        // Smoke mode: injected votes carry no signatures, so the block cannot be
        // proved. It advances the local head only and is never gossiped.
        apply_locally(owner, &plan, tick);
        owner.planned_proposal = Some(plan);
        return out;
    }
    owner.planned_proposal = Some(plan.clone());
    match request_block_proof(owner, tick, plan) {
        Ok(events) => out.extend(events),
        Err(reason) => {
            tracing::info!(slot = tick.slot.get(), %reason, "block proof not requested");
        }
    }
    out
}

/// How many trailing intervals of slot N start the proof for slot N+1.
///
/// A Type-2 of two components is about 3.3 s at leanVM `e2592df4`; one
/// interval (800 ms) of overlap is not enough to land the block at slot start.
pub const EARLY_PROPOSAL_INTERVALS: u64 = 2;

/// True in the last [`EARLY_PROPOSAL_INTERVALS`] of a slot (intervals 3–4
/// when `intervals_per_slot` is 5). Interval 2 stays free for aggregation.
pub fn in_early_proposal_window(owner: &ChainOwner, interval: u8) -> bool {
    let Some(profile) = owner.profile.as_ref() else {
        return false;
    };
    let last = profile.intervals_per_slot.saturating_sub(1);
    let first = last.saturating_sub(EARLY_PROPOSAL_INTERVALS.saturating_sub(1));
    let i = u64::from(interval);
    i >= first && i <= last
}

/// True when this node is the proposer of `tick.slot + 1`.
pub fn proposes_next_slot(owner: &ChainOwner, tick: DutyTick) -> bool {
    let n = owner
        .head_state
        .as_ref()
        .map(|s| s.validators.len() as u64)
        .unwrap_or(0);
    is_assigned_proposer(owner, tick.slot.get() + 1, n)
}

/// Keep the prover free for the next slot's block: it is already proving a
/// block, or this node proposes N+1 and the early-build window has opened.
pub fn prover_reserved_for_block(owner: &ChainOwner, tick: DutyTick) -> bool {
    if owner.local_finality || owner.prover.is_none() {
        return false;
    }
    if owner.prover.as_ref().is_some_and(|p| p.block_in_flight()) {
        return true;
    }
    in_early_proposal_window(owner, tick.interval) && proposes_next_slot(owner, tick)
}

/// During the last two intervals of a slot, start the next slot's block when we
/// propose it. The Type-2 merge takes seconds at leanVM `e2592df4`; starting
/// two intervals early lets it overlap the boundary. Import and gossip still
/// wait until the wall clock reaches the block's slot
/// ([`crate::proof_collect::release_deferred_block`]).
pub fn try_plan_next_slot_proposal(owner: &mut ChainOwner, tick: DutyTick) -> Vec<ChainEvent> {
    if owner.local_finality
        || owner.prover.is_none()
        || !in_early_proposal_window(owner, tick.interval)
    {
        return Vec::new();
    }
    let next = DutyTick {
        slot: Slot::new(tick.slot.get() + 1),
        interval: 0,
        generation: tick.generation,
    };
    try_plan_proposal(owner, next)
}

fn request_block_proof(
    owner: &mut ChainOwner,
    tick: DutyTick,
    mut plan: PlanTransition,
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
    let proofs = std::mem::take(&mut plan.attestation_proofs);
    let mut attestation_proofs = Vec::with_capacity(proofs.len());
    for (attestation, proof) in plan.block.body.attestations.iter().zip(proofs) {
        attestation_proofs.push(KeyedProof {
            public_keys: attestation_keys_for_bits(state, &attestation.aggregation_bits.bits)?,
            proof,
        });
    }
    let root = plan.block_root()?;
    let duty_view = owner.snapshot(tick.slot, 0).duty_view;
    let local = owner
        .proposer_for(plan.block.proposer_index.get())
        .ok_or("no local proposer for the assigned validator")?;
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
    owner.block_proof_slot = Some(tick.slot.get());
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

/// Pre-state for a finished block proof, or why the plan went stale.
///
/// With a live store the block stays publishable while its parent is still in
/// the tree and no block at or past its slot has become head: it is still the
/// only block for our slot. Without a store the head must not have moved.
fn proof_pre_state<'a>(
    owner: &'a ChainOwner,
    plan: &PlanTransition,
) -> Result<&'a ethean_types::State, String> {
    if owner.fc.is_none() {
        if owner.head_root != plan.parent_root {
            return Err("head moved while the block proof was built".into());
        }
        return owner
            .head_state
            .as_ref()
            .ok_or_else(|| "no head state".into());
    }
    let head_slot = owner.head_state.as_ref().map(|s| s.slot.get()).unwrap_or(0);
    if head_slot >= plan.block.slot.get() {
        return Err("a block at or past our slot became head while the proof was built".into());
    }
    owner
        .pre_state_for_parent(plan.parent_root)
        .ok_or_else(|| "parent left the fork-choice tree while the proof was built".into())
}

/// Verify a returned block proof against the parent registry, import the
/// block, and queue it for gossip. Stale plans are dropped.
pub fn accept_block_proof(
    owner: &mut ChainOwner,
    mut plan: PlanTransition,
    proof: Vec<u8>,
    aggregation: std::time::Duration,
) -> Result<Vec<ChainEvent>, String> {
    let (applied, gossip, parent_header_slot, block_bits, committees) = {
        let pre = proof_pre_state(owner, &plan)?;
        let profile = owner.profile.as_ref().ok_or("no profile")?;
        plan.aggregate_proof = proof;
        let signed = assemble_signed_block(&plan)?;
        let ctx = TransitionContext::new(profile.clone());
        let started = Instant::now();
        let applied = apply_block(pre, &signed, &ctx, &LeanMultisigVerifier)
            .map_err(|e| format!("own block rejected: {e}"))?;
        crate::lean_metrics::transition(started.elapsed(), &applied.timings);
        let payloads = plan.block.body.attestations.len() as f64;
        ethean_metrics::lean::observe("lean_block_aggregated_payloads", &[], payloads);
        ethean_metrics::lean::observe(
            "lean_block_building_payload_aggregation_time_seconds",
            &[],
            aggregation.as_secs_f64(),
        );
        let gossip = encode_proposal_gossip(&plan, profile.fork_name)?;
        (
            applied,
            gossip,
            pre.latest_block_header.slot.get(),
            crate::lean_metrics::coverage::union_bits(plan.block.body.attestations.iter()),
            profile.attestation_subnet_count() as u64,
        )
    };
    let root = gossip.block_root;
    let pool_now = crate::lean_metrics::coverage::pool_bits(owner);
    crate::lean_metrics::coverage::record_block_coverage(
        &block_bits,
        &block_bits,
        &pool_now,
        committees,
    );
    for attestation in &plan.block.body.attestations {
        owner.remember_known_payload(
            attestation.data.hash_tree_root(),
            attestation.data.slot.get(),
        );
    }
    let post = applied.post_state;
    let slot = plan.block.slot.get();
    let attestations = plan.block.body.attestations.len();
    let parent_root = plan.parent_root;
    if owner.fc.is_some() {
        owner.fc_on_block(plan.block, post);
    } else {
        owner.head_state = Some(post);
        owner.advance_head(root, parent_root);
    }
    let head_slot = owner.head_state.as_ref().map(|s| s.slot.get()).unwrap_or(0);
    tracing::debug!(
        slot,
        parent_slot = parent_header_slot,
        attestations,
        head_slot,
        is_head = owner.head_root == root,
        "own block proved and imported"
    );
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
            root: owner.head_root,
            slot: head_slot,
        },
    ];
    owner.pending_block_gossip = Some(gossip);
    Ok(events)
}

fn apply_locally(owner: &mut ChainOwner, plan: &PlanTransition, tick: DutyTick) {
    match crate::local_finality::apply_planned_locally(owner, plan.clone()) {
        Ok(applied) => {
            // Persist the solo block for --data-dir resume and range serving. It
            // carries no proof (injected votes are unsigned) and is never gossiped.
            let local = SignedBlock::new(plan.block.clone(), MultiMessageAggregate::default());
            if let Ok(payload) = local.ssz_encode() {
                owner.remember_durable_block(applied, payload);
            }
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
