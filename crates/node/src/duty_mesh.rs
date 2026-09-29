//! Wall / until-signal duty loops that keep QuicSwarm polled for mesh peers.

use crate::client::EtheanClient;
use crate::commands::ChainCommand;
use crate::dispatch::apply_command;
use crate::duty_step::apply_wall_step;
use crate::events::ChainEvent;
use crate::wall_tick::{ms_until_genesis, ms_until_next_interval};
use crate::{Error, Result};
use ethean_genesis::{SystemTimeSource, TimeSource};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Longest single swarm wait between duty intervals; bounds how late a
/// finished proof is noticed.
const IDLE_PUMP_SLICE_MS: u64 = 50;

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
            if self.wait_if_pre_genesis(&time).await? {
                continue;
            }
            let drained = self
                .apply_network_budget(32, Duration::from_millis(20))
                .await?;
            if drained > 0 {
                debug!(drained, "duty network pump");
            }
            let before = events.len();
            events.extend(apply_wall_step(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
            )?);
            crate::api_events::publish_optional(&self.api, &events[before..]);
            self.flush_chain_persist();
            #[cfg(feature = "libp2p-quic")]
            {
                let gossip = self.flush_pending_events()?;
                crate::api_events::publish_optional(&self.api, &gossip);
                for ev in gossip {
                    events.push(ev);
                }
            }
            let _ = self.refresh_slot_metrics();
            if enable_sleep && i + 1 < ticks {
                events.extend(self.pump_until_next_interval(&time).await?);
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
        if self.wait_if_pre_genesis(time).await? {
            return Ok(Vec::new());
        }
        #[cfg_attr(not(feature = "libp2p-quic"), allow(unused_mut))]
        let mut step_events = apply_wall_step(
            &self.clock,
            &mut self.owner,
            &mut self.shutdown,
            &mut self.sync,
        )?;
        self.flush_chain_persist();
        #[cfg(feature = "libp2p-quic")]
        {
            for ev in self.flush_pending_events()? {
                step_events.push(ev);
            }
        }
        crate::api_events::publish_optional(&self.api, &step_events);
        let _ = self.refresh_slot_metrics();
        if enable_sleep && self.shutdown.accepts_new_duties() {
            step_events.extend(self.pump_until_next_interval(time).await?);
        }
        Ok(step_events)
    }

    /// Keep the swarm polled until the next duty interval and pick up proofs
    /// the moment they finish. Sleeping through the interval would hold a
    /// finished block proof (and inbound gossip) for up to one interval.
    async fn pump_until_next_interval(
        &mut self,
        time: &SystemTimeSource,
    ) -> Result<Vec<ChainEvent>> {
        let mut events = Vec::new();
        let now_ms = time.unix_millis().map_err(Error::Clock)?;
        let deadline_ms = now_ms + ms_until_next_interval(&self.clock, now_ms)?;
        loop {
            let now_ms = time.unix_millis().map_err(Error::Clock)?;
            if now_ms >= deadline_ms || !self.shutdown.accepts_new_duties() {
                break;
            }
            let slice = Duration::from_millis((deadline_ms - now_ms).min(IDLE_PUMP_SLICE_MS));
            if self.has_swarm() {
                let deadline = tokio::time::Instant::now() + slice;
                self.apply_network_window(64, slice, Some(deadline)).await?;
            } else {
                tokio::time::sleep(slice).await;
            }
            #[cfg_attr(not(feature = "libp2p-quic"), allow(unused_mut))]
            let mut ready = crate::proof_collect::collect_proofs(&mut self.owner);
            if !ready.is_empty() {
                self.flush_chain_persist();
            }
            #[cfg(feature = "libp2p-quic")]
            ready.extend(self.flush_pending_events()?);
            if !ready.is_empty() {
                debug!(n = ready.len(), "events produced between duty intervals");
                crate::api_events::publish_optional(&self.api, &ready);
                events.extend(ready);
            }
        }
        Ok(events)
    }

    fn has_swarm(&self) -> bool {
        #[cfg(feature = "libp2p-quic")]
        {
            self.swarm.is_some()
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            false
        }
    }

    /// Before genesis there are no duties: keep the swarm pumped and the HTTP
    /// snapshot fresh, nap (at most one second per round) and report `true`.
    async fn wait_if_pre_genesis(&mut self, time: &SystemTimeSource) -> Result<bool> {
        let now_ms = time.unix_millis().map_err(Error::Clock)?;
        let Some(remaining) = ms_until_genesis(&self.clock, now_ms)? else {
            return Ok(false);
        };
        if !self.pre_genesis_logged {
            info!(
                remaining_ms = remaining,
                "waiting for genesis before running duties"
            );
            self.pre_genesis_logged = true;
        }
        let _ = self.refresh_slot_metrics();
        tokio::time::sleep(Duration::from_millis(remaining.min(1_000))).await;
        Ok(true)
    }

    #[cfg(feature = "libp2p-quic")]
    fn flush_pending_events(&mut self) -> Result<Vec<ChainEvent>> {
        let Some(facade) = self.swarm.as_mut() else {
            return Ok(Vec::new());
        };
        crate::swarm_pump::flush_all_pending_gossip(facade, &mut self.owner)
    }
}
