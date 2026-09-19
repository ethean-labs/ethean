//! Wall-clock duty loop until Ctrl-C / process termination signal.

use crate::chain_owner::ChainOwner;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::duty_step::apply_wall_step;
use crate::events::ChainEvent;
use crate::shutdown::ShutdownState;
use crate::wall_tick::ms_until_next_interval;
use crate::{Error, Result};
use ethean_genesis::{SlotClock, SystemTimeSource, TimeSource};
use ethean_sync::SyncStatus;
use std::time::Duration;
use tracing::{info, warn};

/// Drive duties until Ctrl-C (or equivalent) then drain shutdown.
pub async fn run_until_signal(
    clock: &SlotClock,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
    enable_sleep: bool,
) -> Result<Vec<ChainEvent>> {
    let mut events = Vec::new();
    let time = SystemTimeSource;
    info!("Duty loop running until Ctrl-C");

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        if !shutdown.accepts_new_duties() {
            break;
        }

        tokio::select! {
            _ = &mut ctrl_c => {
                info!("Ctrl-C received; draining duties");
                break;
            }
            step = run_one_step(clock, owner, shutdown, sync, enable_sleep, &time) => {
                match step {
                    Ok(step_events) => events.extend(step_events),
                    Err(e) => {
                        warn!(error = %e, "Duty step failed; shutting down");
                        break;
                    }
                }
            }
        }
    }

    events.push(apply_command(
        owner,
        shutdown,
        ChainCommand::Shutdown,
    ));
    Ok(events)
}

async fn run_one_step(
    clock: &SlotClock,
    owner: &mut ChainOwner,
    shutdown: &mut ShutdownState,
    sync: &mut SyncStatus,
    enable_sleep: bool,
    time: &SystemTimeSource,
) -> Result<Vec<ChainEvent>> {
    let step_events = apply_wall_step(clock, owner, shutdown, sync)?;
    if enable_sleep && shutdown.accepts_new_duties() {
        let now_ms = time.unix_millis().map_err(Error::Clock)?;
        let wait = ms_until_next_interval(clock, now_ms)?;
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }
    Ok(step_events)
}
