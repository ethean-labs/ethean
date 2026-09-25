//! Finite duty-tick loop for smoke and local orchestration.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;
use ethean_profile::ChainProfile;
use ethean_sync::SyncStatus;
use ethean_validator::{evaluate_gate, tick_from_elapsed_ms};

/// Limits for a local smoke / catch-up duty run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DutyLoopConfig {
    /// Maximum ticks to process before forced drain.
    pub max_ticks: u32,
    /// Elapsed milliseconds since genesis at the first tick.
    pub start_elapsed_ms: u64,
}

impl Default for DutyLoopConfig {
    fn default() -> Self {
        Self {
            max_ticks: 5,
            start_elapsed_ms: 0,
        }
    }
}

/// Drive ticks through the owner, sync hysteresis, and duty gate.
///
/// Uses profile interval length to advance elapsed time (no wall sleep).
pub fn run_duty_loop(
    profile: &ChainProfile,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
    cfg: DutyLoopConfig,
) -> Vec<ChainEvent> {
    let mut events = Vec::new();
    let mut elapsed = cfg.start_elapsed_ms;
    let generation = owner.generation.max(1);
    let interval_ms = profile.milliseconds_per_interval.max(1);

    for _ in 0..cfg.max_ticks {
        if !shutdown.accepts_new_duties() {
            break;
        }

        let tick = tick_from_elapsed_ms(elapsed, profile, generation);
        let local_head = owner
            .head_state
            .as_ref()
            .map(|s| s.slot)
            .unwrap_or(tick.slot);
        sync.observe_local(local_head);
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
            }
        }

        elapsed = elapsed.saturating_add(interval_ms);
    }

    events.push(apply_command(owner, shutdown, ChainCommand::Shutdown));
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_profile::lstar_devnet;
    use ethean_types::State;
    use ethean_validator::SYNC_LAG_THRESHOLD_SLOTS;

    #[test]
    fn smoke_loop_accepts_ticks_then_shuts_down() {
        let profile = lstar_devnet().expect("lstar");
        let mut owner = ChainOwner::new(SYNC_LAG_THRESHOLD_SLOTS);
        owner.generation = 1;
        owner.head_state = Some(State::default());
        let mut shutdown = ShutdownState::default();
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        let events = run_duty_loop(
            &profile,
            &mut owner,
            &mut shutdown,
            &mut sync,
            DutyLoopConfig {
                max_ticks: 3,
                start_elapsed_ms: 0,
            },
        );
        assert!(events
            .iter()
            .any(|e| matches!(e, ChainEvent::TickAccepted(_))));
        assert_eq!(events.last(), Some(&ChainEvent::ShutdownComplete));
        assert!(!shutdown.accepts_new_duties());
    }

    #[test]
    fn duty_ticks_preserve_status_peer_horizon() {
        let profile = lstar_devnet().expect("lstar");
        let mut owner = ChainOwner::new(SYNC_LAG_THRESHOLD_SLOTS);
        owner.generation = 1;
        owner.head_state = Some(State::default());
        let mut shutdown = ShutdownState::default();
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        sync.observe(Slot::new(0), Slot::new(40));
        assert!(!sync.duties_allowed());
        let _ = run_duty_loop(
            &profile,
            &mut owner,
            &mut shutdown,
            &mut sync,
            DutyLoopConfig {
                max_ticks: 3,
                start_elapsed_ms: 0,
            },
        );
        assert_eq!(sync.peer_horizon.get(), 40);
        assert!(!sync.duties_allowed());
    }
}
