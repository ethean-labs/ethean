//! Wall-clock duty loop with optional interval sleep (tokio).

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;
use crate::wall_tick::{ms_until_next_interval, tick_from_wall};
use crate::{Error, Result};
use ethean_genesis::{SlotClock, SystemTimeSource, TimeSource};
use ethean_sync::SyncStatus;
use ethean_validator::evaluate_gate;
use std::time::Duration;
use tracing::debug;

/// Configuration for [`run_wall_duty_loop`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallLoopConfig {
    /// Maximum wall samples to apply before forced drain.
    pub max_ticks: u32,
    /// Sleep until the next interval when true.
    pub enable_sleep: bool,
}

impl Default for WallLoopConfig {
    fn default() -> Self {
        Self {
            max_ticks: 5,
            enable_sleep: true,
        }
    }
}

/// Drive duties from the system wall clock.
pub async fn run_wall_duty_loop(
    clock: &SlotClock,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
    cfg: WallLoopConfig,
) -> Result<Vec<ChainEvent>> {
    let mut events = Vec::new();
    let generation = owner.generation.max(1);
    let time = SystemTimeSource;

    for i in 0..cfg.max_ticks {
        if !shutdown.accepts_new_duties() {
            break;
        }

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
            }
        }

        if cfg.enable_sleep && i + 1 < cfg.max_ticks {
            let now_ms = time.unix_millis().map_err(Error::Clock)?;
            let wait = ms_until_next_interval(clock, now_ms)?;
            debug!(wait_ms = wait, "Sleeping until next duty interval");
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }
    }

    events.push(apply_command(
        owner,
        shutdown,
        ChainCommand::Shutdown,
    ));
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_genesis::local_smoke_genesis;
    use ethean_primitives::Slot;
    use ethean_profile::lstar_devnet;
    use ethean_types::State;

    #[tokio::test]
    async fn wall_loop_no_sleep_completes() {
        let profile = lstar_devnet().unwrap();
        let built = local_smoke_genesis(1_700_000_000).unwrap();
        let clock = SlotClock::new(built.state.genesis_time(), profile).unwrap();
        let mut owner = ChainOwner::new(32);
        owner.generation = 1;
        owner.head_state = Some(State::default());
        let mut shutdown = ShutdownState::default();
        let mut sync = SyncStatus::new(Slot::new(0), Slot::new(0));
        let events = run_wall_duty_loop(
            &clock,
            &mut owner,
            &mut shutdown,
            &mut sync,
            WallLoopConfig {
                max_ticks: 2,
                enable_sleep: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(events.last(), Some(&ChainEvent::ShutdownComplete));
    }
}
