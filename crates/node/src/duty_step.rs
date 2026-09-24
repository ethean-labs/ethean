//! Shared single-step wall duty application (no sleep).

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::duty_propose::try_plan_proposal;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;
use crate::wall_tick::tick_from_wall;
use crate::Result;
use ethean_genesis::{SlotClock, SystemTimeSource};
use ethean_sync::SyncStatus;
use ethean_validator::evaluate_gate;

/// Apply one wall-clock duty tick; returns events produced this step.
pub fn apply_wall_step(
    clock: &SlotClock,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
) -> Result<Vec<ChainEvent>> {
    let mut events = crate::proof_collect::collect_proofs(owner);
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
    if was_accepted {
        crate::lean_metrics::tick();
        owner.fc_on_tick(tick.slot.get(), tick.interval, tick.interval == 0);
    }
    events.push(accepted);
    if was_accepted {
        let lag = sync.lag();
        let snap = owner.snapshot(tick.slot, lag);
        if let Err(reason) = evaluate_gate(&snap.duty_view) {
            events.push(ChainEvent::DutySuppressed { tick, reason });
        } else {
            events.extend(crate::duty_attest::try_local_attest(owner, tick));
            events.extend(crate::aggregation_duty::schedule_aggregations(owner));
            events.extend(try_plan_proposal(owner, tick));
        }
    }
    Ok(events)
}
