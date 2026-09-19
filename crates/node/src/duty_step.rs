//! Shared single-step wall duty application (no sleep).

use crate::block_builder::{decide_publish, plan_from_pool, PublishDecision};
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
        } else if let Some(ev) = try_plan_proposal(owner, tick) {
            events.push(ev);
        }
    }
    Ok(events)
}

fn try_plan_proposal(owner: &mut ChainOwner, tick: ethean_validator::DutyTick) -> Option<ChainEvent> {
    let min_slot = tick.slot.get().saturating_sub(owner.max_head_lag_slots);
    owner.aggregates.prune_before(min_slot);

    let (pre, profile) = match (owner.head_state.clone(), owner.profile.clone()) {
        (Some(s), Some(p)) => (s, p),
        _ => return None,
    };
    let n = pre.validators.len() as u64;
    if n == 0 {
        return None;
    }
    let proposer = ValidatorIndex::new(tick.slot.get() % n);
    let plan = plan_from_pool(
        &owner.aggregates,
        owner.head_root,
        tick.slot,
        proposer,
        &pre,
        profile,
        16,
    )
    .ok()?;
    let root = plan.block_root().ok()?;
    let attestations = plan.block.body.attestations.len();
    let publish_allowed = matches!(
        decide_publish(tick, tick, true),
        PublishDecision::Allow
    ) && tick.interval == 0;

    owner.planned_proposal = Some(plan);
    owner.planned_tick = Some(tick);
    Some(ChainEvent::ProposalPlanned {
        root,
        slot: tick.slot.get(),
        attestations,
        publish_allowed,
    })
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
    fn plans_proposal_when_head_state_ready() {
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
        let ev = try_plan_proposal(&mut owner, tick).expect("plan event");
        assert!(matches!(
            ev,
            ChainEvent::ProposalPlanned {
                publish_allowed: true,
                attestations: 0,
                ..
            }
        ));
        assert!(owner.planned_proposal.is_some());
        assert_eq!(owner.planned_tick, Some(tick));
    }
}
