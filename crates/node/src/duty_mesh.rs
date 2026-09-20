//! Wall / until-signal duty loops that keep QuicSwarm polled for mesh peers.

use crate::client::EtheanClient;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::duty_step::apply_wall_step;
use crate::events::ChainEvent;
use crate::wall_tick::ms_until_next_interval;
use crate::{Error, Result};
use ethean_genesis::{SystemTimeSource, TimeSource};
use std::time::Duration;
use tracing::{debug, info, warn};

impl EtheanClient {
    /// Wall-clock duty loop that pumps the swarm then flushes gossip each tick.
    pub(crate) async fn run_wall_with_flush(
        &mut self,
        ticks: u32,
        enable_sleep: bool,
    ) -> Result<Vec<ChainEvent>> {
        let mut events = Vec::new();
        let time = SystemTimeSource;
        for i in 0..ticks {
            if !self.shutdown.accepts_new_duties() {
                break;
            }
            let drained = self
                .apply_network_budget(32, Duration::from_millis(20))
                .await?;
            if drained > 0 {
                debug!(drained, "duty network pump");
            }
            events.extend(apply_wall_step(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
            )?);
            self.flush_chain_persist();
            #[cfg(feature = "libp2p-quic")]
            if let Some(ev) = self.flush_pending_event()? {
                events.push(ev);
            }
            let _ = self.refresh_slot_metrics();
            if enable_sleep && i + 1 < ticks {
                let now_ms = time.unix_millis().map_err(Error::Clock)?;
                let wait = ms_until_next_interval(&self.clock, now_ms)?;
                debug!(wait_ms = wait, "Sleeping until next duty interval");
                tokio::time::sleep(Duration::from_millis(wait)).await;
            }
        }
        events.push(apply_command(
            &mut self.owner,
            &mut self.shutdown,
            ChainCommand::Shutdown,
        ));
        Ok(events)
    }

    /// Until-signal duty loop that keeps QuicSwarm polled between wall samples.
    pub(crate) async fn run_until_signal_with_flush(
        &mut self,
        enable_sleep: bool,
    ) -> Result<Vec<ChainEvent>> {
        let mut events = Vec::new();
        let time = SystemTimeSource;
        info!("Duty loop running until Ctrl-C");
        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        loop {
            if !self.shutdown.accepts_new_duties() {
                break;
            }
            tokio::select! {
                _ = &mut ctrl_c => {
                    info!("Ctrl-C received; draining duties");
                    break;
                }
                step = self.one_mesh_step(enable_sleep, &time) => {
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
            &mut self.owner,
            &mut self.shutdown,
            ChainCommand::Shutdown,
        ));
        Ok(events)
    }

    async fn one_mesh_step(
        &mut self,
        enable_sleep: bool,
        time: &SystemTimeSource,
    ) -> Result<Vec<ChainEvent>> {
        let _ = self
            .apply_network_budget(32, Duration::from_millis(20))
            .await?;
        let mut step_events = apply_wall_step(
            &self.clock,
            &mut self.owner,
            &mut self.shutdown,
            &mut self.sync,
        )?;
        self.flush_chain_persist();
        #[cfg(feature = "libp2p-quic")]
        {
            if let Some(ev) = self.flush_pending_event()? {
                step_events.push(ev);
            }
        }
        let _ = self.refresh_slot_metrics();
        if enable_sleep && self.shutdown.accepts_new_duties() {
            let now_ms = time.unix_millis().map_err(Error::Clock)?;
            let wait = ms_until_next_interval(&self.clock, now_ms)?;
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }
        Ok(step_events)
    }

    #[cfg(feature = "libp2p-quic")]
    fn flush_pending_event(&mut self) -> Result<Option<ChainEvent>> {
        let Some(facade) = self.swarm.as_mut() else {
            return Ok(None);
        };
        crate::swarm_pump::flush_pending_event(facade, &mut self.owner)
    }
}
