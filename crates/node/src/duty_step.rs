//! Shared single-step wall duty application (no sleep).

use crate::block_builder::{
    decide_publish, encode_proposal_gossip, plan_from_pool, try_attach_type2_proof,
    PublishDecision, Type2ProveResult,
};
use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;
use crate::wall_tick::tick_from_wall;
use crate::Result;
use ethean_genesis::{SlotClock, SystemTimeSource};
use ethean_primitives::ValidatorIndex;
use ethean_sync::SyncStatus;
use ethean_validator::evaluate_gate;

/// Apply one wall-clock duty tick; returns events produced this step.
pub fn apply_wall_step(
    clock: &SlotClock,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
) -> Result<Vec<ChainEvent>> {
    let mut events = Vec::new();
    if !shutdown.accepts_new_duties() {
        return Ok(events);
    }
    let generation = owner.generation.max(1);
    let time = SystemTimeSource;
    let tick = tick_from_wall(clock, &time, generation)?;
    sync.observe(tick.slot, tick.slot);
    let syncing = !sync.duties_allowed();
    events.push(apply_command(
        owner,
        shutdown,
        ChainCommand::SetSyncing(syncing),
    ));
    let accepted = apply_command(owner, shutdown, ChainCommand::Tick(tick));
    let was_accepted = matches!(accepted, ChainEvent::TickAccepted(_));
    events.push(accepted);
    if was_accepted {
        let lag = sync.lag();
        let snap = owner.snapshot(tick.slot, lag);
        if let Err(reason) = evaluate_gate(&snap.duty_view) {
            events.push(ChainEvent::DutySuppressed { tick, reason });
        } else {
            events.extend(crate::duty_aggregator::evaluate_aggregator_duties(
                owner, tick, lag,
            ));
            events.extend(try_plan_proposal(owner, tick));
        }
    }
    Ok(events)
}

fn try_plan_proposal(
    owner: &mut ChainOwner,
    tick: ethean_validator::DutyTick,
) -> Vec<ChainEvent> {
    let mut out = Vec::new();
    let min_slot = tick.slot.get().saturating_sub(owner.max_head_lag_slots);
    owner.aggregates.prune_before(min_slot);

    let (pre, profile) = match (owner.head_state.clone(), owner.profile.clone()) {
        (Some(s), Some(p)) => (s, p),
        _ => return out,
    };
    let n = pre.validators.len() as u64;
    if n == 0 {
        return out;
    }
    let proposer = ValidatorIndex::new(tick.slot.get() % n);
    let mut plan = match plan_from_pool(
        &owner.aggregates,
        owner.head_root,
        tick.slot,
        proposer,
        &pre,
        profile.clone(),
        16,
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(
                error = %e,
                slot = tick.slot.get(),
                head_slot = pre.slot.get(),
                "proposal plan skipped"
            );
            return out;
        }
    };

    if owner.local_finality || owner.is_aggregator {
        crate::local_finality::inject_local_aggregate(owner, &mut plan);
        if let Err(e) = crate::local_finality::rebind_plan_state_root(owner, &mut plan) {
            tracing::debug!(error = %e, "local attestation rebind skipped");
        }
    }

    let attestations = plan.block.body.attestations.len();
    // Interval 0 is the normal publish window. Local-finality also allows the
    // first tick of a strictly future slot so solo runs do not stall if the
    // wall sampler skips interval 0 after a long sleep.
    let future_slot = tick.slot.get() > pre.slot.get();
    let publish_allowed = matches!(decide_publish(tick, tick, true), PublishDecision::Allow)
        && future_slot
        && (tick.interval == 0 || owner.local_finality);

    if publish_allowed {
        if let Type2ProveResult::Attached { proof_len } = try_attach_type2_proof(&mut plan) {
            let root = plan.block_root().unwrap_or([0u8; 32]);
            out.push(ChainEvent::Type2ProofAttached { root, proof_len });
        }
    }

    let root = match plan.block_root() {
        Ok(r) => r,
        Err(_) => return out,
    };

    let duty_view = owner.snapshot(tick.slot, 0).duty_view;
    let mut binding_ok = true;
    if publish_allowed {
        if let Some(local) = owner.proposer.as_mut() {
            match local.sign_proposal(tick, &duty_view, plan.parent_root, root) {
                Ok(sig) => {
                    let signature_len = sig.len();
                    match local.verify_proposal(tick, root, &sig) {
                        Ok(()) => {
                            plan.proposer_signature = Some(sig);
                            out.push(ChainEvent::ProposalSigned { root, signature_len });
                            out.push(ChainEvent::ProposalBindingVerified { root });
                        }
                        Err(_) => {
                            binding_ok = false;
                            plan.proposer_signature = None;
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    owner.planned_proposal = Some(plan.clone());
    owner.planned_tick = Some(tick);
    out.push(ChainEvent::ProposalPlanned {
        root,
        slot: tick.slot.get(),
        attestations,
        publish_allowed,
    });

    if publish_allowed && binding_ok {
        if let Ok(gossip) = encode_proposal_gossip(&plan, profile.fork_name) {
            let ready = ChainEvent::ProposalGossipReady {
                root: gossip.block_root,
                topic: gossip.topic.clone(),
                payload_len: gossip.payload.len(),
                has_type2_proof: gossip.has_type2_proof,
                proposer_sig_len: gossip.proposer_sig_len,
            };
            owner.pending_block_gossip = Some(gossip);
            out.push(ready);
        }
    } else if !owner.local_finality {
        owner.pending_block_gossip = None;
    }

    // Proposer self-import so solo / local-finality runs advance head without peers.
    // Allowed even when binding fails (unsigned local smoke).
    if publish_allowed && owner.local_finality {
        match crate::local_finality::apply_planned_locally(owner, &plan) {
            Ok(applied) => {
                crate::local_finality::promote_local_checkpoints(owner, applied);
                tracing::info!(
                    slot = tick.slot.get(),
                    root = %format!("{:02x}{:02x}…", applied[0], applied[1]),
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
    } else if !publish_allowed {
        owner.pending_block_gossip = None;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_proposer::LocalProposer;
    use ethean_primitives::{Bytes52, Slot, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_transition::process_slots;
    use ethean_types::{BlockHeader, Checkpoint, GenesisConfig, State, Validator};
    use ethean_validator::DutyTick;

    fn sample_state(validators: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..validators {
            vals.push(
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64))
                    .unwrap(),
            );
        }
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vals,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn plans_signs_and_encodes_gossip_when_publish_allowed() {
        let pre = sample_state(3);
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent = advanced.latest_block_header.hash_tree_root();
        assert_ne!(parent, HASH32_ZERO);

        let mut owner = ChainOwner::new(2);
        owner.head_root = parent;
        owner.head_state = Some(pre);
        owner.profile = Some(lstar_devnet().unwrap());
        owner.proposer = Some(LocalProposer::smoke().expect("proposer"));
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        let events = try_plan_proposal(&mut owner, tick);
        assert!(events
            .iter()
            .any(|e| matches!(e, ChainEvent::Type2ProofAttached { .. })));
        assert!(events.iter().any(|e| matches!(e, ChainEvent::ProposalSigned { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, ChainEvent::ProposalBindingVerified { .. })));
        assert!(matches!(
            events.iter().find(|e| matches!(e, ChainEvent::ProposalPlanned { .. })),
            Some(ChainEvent::ProposalPlanned {
                publish_allowed: true,
                attestations: 0,
                ..
            })
        ));
        assert!(matches!(
            events.iter().find(|e| matches!(e, ChainEvent::ProposalGossipReady { .. })),
            Some(ChainEvent::ProposalGossipReady {
                has_type2_proof: true,
                proposer_sig_len,
                topic,
                ..
            }) if *proposer_sig_len == ethean_crypto::SIGNATURE_BYTES
                && topic.ends_with("/block/ssz_snappy")
        ));
        let pending = owner.pending_block_gossip.expect("pending");
        assert!(pending.has_type2_proof);
        assert_eq!(pending.proposer_sig_len, ethean_crypto::SIGNATURE_BYTES);
        assert!(owner
            .planned_proposal
            .as_ref()
            .unwrap()
            .proposer_signature
            .is_some());
    }
}
