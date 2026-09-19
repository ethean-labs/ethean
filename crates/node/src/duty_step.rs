//! Shared single-step wall duty application (no sleep).

use crate::block_builder::{
    decide_publish, encode_proposal_gossip, plan_from_pool, PublishDecision,
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
    let plan = match plan_from_pool(
        &owner.aggregates,
        owner.head_root,
        tick.slot,
        proposer,
        &pre,
        profile.clone(),
        16,
    ) {
        Ok(p) => p,
        Err(_) => return out,
    };
    let root = match plan.block_root() {
        Ok(r) => r,
        Err(_) => return out,
    };
    let attestations = plan.block.body.attestations.len();
    let publish_allowed = matches!(decide_publish(tick, tick, true), PublishDecision::Allow)
        && tick.interval == 0;

    owner.planned_proposal = Some(plan.clone());
    owner.planned_tick = Some(tick);
    out.push(ChainEvent::ProposalPlanned {
        root,
        slot: tick.slot.get(),
        attestations,
        publish_allowed,
    });

    if publish_allowed {
        if let Ok(gossip) = encode_proposal_gossip(&plan, profile.fork_name) {
            let ready = ChainEvent::ProposalGossipReady {
                root: gossip.block_root,
                topic: gossip.topic.clone(),
                payload_len: gossip.payload.len(),
                has_type2_proof: gossip.has_type2_proof,
            };
            owner.pending_block_gossip = Some(gossip);
            out.push(ready);
        }
    } else {
        owner.pending_block_gossip = None;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn plans_and_encodes_gossip_when_publish_allowed() {
        let pre = sample_state(3);
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent = advanced.latest_block_header.hash_tree_root();
        assert_ne!(parent, HASH32_ZERO);

        let mut owner = ChainOwner::new(2);
        owner.head_root = parent;
        owner.head_state = Some(pre);
        owner.profile = Some(lstar_devnet().unwrap());
        let tick = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        let events = try_plan_proposal(&mut owner, tick);
        assert!(matches!(
            events[0],
            ChainEvent::ProposalPlanned {
                publish_allowed: true,
                attestations: 0,
                ..
            }
        ));
        assert!(matches!(
            &events[1],
            ChainEvent::ProposalGossipReady {
                has_type2_proof: false,
                topic,
                ..
            } if topic.ends_with("/block/ssz_snappy")
        ));
        let pending = owner.pending_block_gossip.expect("pending");
        assert!(!pending.payload.is_empty());
        assert_eq!(owner.planned_tick, Some(tick));
    }
}
