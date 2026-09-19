//! QuicSwarm accessors and duty-loop flush hooks for [`EtheanClient`].

use crate::client::EtheanClient;
use crate::events::ChainEvent;
use crate::signal_loop::run_until_signal;
use crate::wall_loop::{run_wall_duty_loop, WallLoopConfig};
use crate::Result;

impl EtheanClient {
    /// Wall-clock duty loop that flushes pending block gossip after each tick.
    pub(crate) async fn run_wall_with_flush(
        &mut self,
        ticks: u32,
        enable_sleep: bool,
    ) -> Result<Vec<ChainEvent>> {
        let cfg = WallLoopConfig {
            max_ticks: ticks,
            enable_sleep,
        };
        #[cfg(feature = "libp2p-quic")]
        {
            let swarm = &mut self.swarm;
            return run_wall_duty_loop(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
                cfg,
                |owner| match swarm.as_mut() {
                    Some(facade) => crate::swarm_pump::flush_pending_event(facade, owner),
                    None => Ok(None),
                },
            )
            .await;
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            run_wall_duty_loop(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
                cfg,
                |_| Ok(None),
            )
            .await
        }
    }

    /// Until-signal duty loop that flushes pending block gossip after each tick.
    pub(crate) async fn run_until_signal_with_flush(
        &mut self,
        enable_sleep: bool,
    ) -> Result<Vec<ChainEvent>> {
        #[cfg(feature = "libp2p-quic")]
        {
            let swarm = &mut self.swarm;
            return run_until_signal(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
                enable_sleep,
                |owner| match swarm.as_mut() {
                    Some(facade) => crate::swarm_pump::flush_pending_event(facade, owner),
                    None => Ok(None),
                },
            )
            .await;
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            run_until_signal(
                &self.clock,
                &mut self.owner,
                &mut self.shutdown,
                &mut self.sync,
                enable_sleep,
                |_| Ok(None),
            )
            .await
        }
    }
}

#[cfg(feature = "libp2p-quic")]
impl EtheanClient {
    /// Bound libp2p QUIC facade when boot completed.
    pub fn swarm(&self) -> Option<&crate::network::SwarmFacade> {
        self.swarm.as_ref()
    }

    /// Mutable access for dial / event pump (gossip loop ownership).
    pub fn swarm_mut(&mut self) -> Option<&mut crate::network::SwarmFacade> {
        self.swarm.as_mut()
    }

    /// Drain a small budget of QuicSwarm events and return accepted gossip.
    pub async fn pump_network(
        &mut self,
        max_events: u32,
    ) -> Result<crate::swarm_pump::PumpBudgetResult> {
        let Some(facade) = self.swarm.as_mut() else {
            return Ok(crate::swarm_pump::PumpBudgetResult::default());
        };
        crate::swarm_pump::pump_swarm_budget(
            facade,
            max_events,
            std::time::Duration::from_millis(2),
        )
        .await
    }

    /// Boot-time pump: queue Status handshakes and ingest accepted gossip.
    pub(crate) async fn boot_pump_status_and_gossip(&mut self) -> Result<()> {
        use tracing::info;
        let budget = self.pump_network(8).await?;
        if budget.drained > 0 {
            info!(
                drained = budget.drained,
                accepted = budget.accepted.len(),
                connected = budget.connected_peers.len(),
                "QuicSwarm pump drained boot events"
            );
        }
        if let Some(local) = self.local_status.clone() {
            let queued = crate::status_handshake::queue_peers(
                &mut self.status_sessions,
                &local,
                &budget.connected_peers,
            );
            if queued > 0 {
                info!(
                    queued,
                    pending = self.status_sessions.pending_len(),
                    "Status handshakes queued for connected peers"
                );
            }
            if let Some(facade) = self.swarm.as_mut() {
                match ethean_network::prepare_status_outbounds(
                    &self.status_sessions,
                    &mut facade.requests,
                ) {
                    Ok(reqs) if !reqs.is_empty() => {
                        let n = reqs.len();
                        facade.enqueue_status_outbounds(reqs);
                        match facade.flush_status_outbox() {
                            Ok(sent) => info!(
                                staged = n,
                                sent,
                                "Status outbound payloads flushed to req/resp"
                            ),
                            Err(e) => info!(
                                staged = n,
                                error = %e,
                                "Status outbox staged; flush deferred"
                            ),
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        info!(error = %e, "failed to stage Status outbounds");
                    }
                }
            }
        }
        let ingest = crate::gossip_ingest::ingest_accepted(
            &mut self.owner,
            &mut self.shutdown,
            &budget.accepted,
        );
        if !ingest.is_empty() {
            info!(n = ingest.len(), "Ingested gossip from boot pump");
        }
        Ok(())
    }

    /// Publish `pending_block_gossip` on the bound QuicSwarm when present.
    pub fn flush_pending_block_gossip(
        &mut self,
    ) -> Result<Option<crate::swarm_pump::PublishedBlock>> {
        let Some(facade) = self.swarm.as_mut() else {
            return Ok(None);
        };
        crate::swarm_pump::publish_pending_block(facade, &mut self.owner)
    }
}
