//! Apply QuicSwarm pump budgets to Status + gossip during duties.

use crate::client::EtheanClient;
use crate::Result;
#[cfg(feature = "libp2p-quic")]
use tracing::info;

impl EtheanClient {
    /// Drain swarm events and drive Status / gossip side effects.
    /// Returns events drained (0 when QuicSwarm is absent).
    pub(crate) async fn apply_network_budget(
        &mut self,
        max_events: u32,
        idle: std::time::Duration,
    ) -> Result<u32> {
        #[cfg(feature = "libp2p-quic")]
        {
            let budget = self.pump_network_idle(max_events, idle).await?;
            let drained = budget.drained;
            self.drive_status_and_gossip(&budget)?;
            return Ok(drained);
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            let _ = (max_events, idle);
            let _ = self.status_sessions.pending_len();
            Ok(0)
        }
    }

    #[cfg(feature = "libp2p-quic")]
    fn drive_status_and_gossip(
        &mut self,
        budget: &crate::swarm_pump::PumpBudgetResult,
    ) -> Result<()> {
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
                                sent, "Status outbound payloads flushed to req/resp"
                            ),
                            Err(e) => info!(
                                staged = n,
                                error = %e,
                                "Status outbox staged; flush deferred"
                            ),
                        }
                    }
                    Ok(_) => {}
                    Err(e) => info!(error = %e, "failed to stage Status outbounds"),
                }
            }
        }
        let ingest = crate::gossip_ingest::ingest_accepted(
            &mut self.owner,
            &mut self.shutdown,
            &budget.accepted,
        );
        if !ingest.is_empty() {
            info!(n = ingest.len(), "Ingested gossip from network pump");
        }
        for (peer, payload) in &budget.status_responses {
            let Some(facade) = self.swarm.as_mut() else {
                break;
            };
            match crate::status_handshake::complete_status_handshake(
                &mut self.status_sessions,
                &mut self.sync,
                &self.owner,
                *peer,
                payload,
                &mut facade.requests,
            ) {
                Ok(out) => {
                    let mut staged = false;
                    if let Some(blocks_req) = out.blocks_by_root {
                        staged = true;
                        facade.enqueue_blocks_outbounds(vec![blocks_req]);
                        match facade.flush_blocks_outbox() {
                            Ok(sent) => info!(
                                peer0 = peer[0],
                                sent, "blocks-by-root flushed after Status response"
                            ),
                            Err(e) => info!(
                                peer0 = peer[0],
                                error = %e,
                                "blocks-by-root staged; flush deferred"
                            ),
                        }
                    }
                    if let Some(range_req) = out.blocks_by_range {
                        staged = true;
                        facade.enqueue_blocks_range_outbounds(vec![range_req]);
                        match facade.flush_blocks_range_outbox() {
                            Ok(sent) => info!(
                                peer0 = peer[0],
                                sent, "blocks-by-range flushed after Status response"
                            ),
                            Err(e) => info!(
                                peer0 = peer[0],
                                error = %e,
                                "blocks-by-range staged; flush deferred"
                            ),
                        }
                    }
                    if !staged {
                        info!(peer0 = peer[0], "Status handshake completed; heads match");
                    }
                }
                Err(e) => {
                    info!(peer0 = peer[0], error = %e, "Status handshake failed");
                }
            }
        }
        for (peer, payload) in budget
            .blocks_by_root_responses
            .iter()
            .chain(budget.blocks_by_range_responses.iter())
        {
            let outcome = crate::blocks_sync::ingest_blocks_by_root_response(
                &mut self.owner,
                &mut self.shutdown,
                self.swarm.as_mut(),
                *peer,
                payload,
            );
            if !outcome.events.is_empty() {
                info!(
                    peer0 = peer[0],
                    n = outcome.events.len(),
                    orphans = self.owner.sync_orphans.len(),
                    "blocks sync response ingested"
                );
            }
            if outcome.fetch_roots.is_empty() {
                continue;
            }
            let Some(facade) = self.swarm.as_mut() else {
                continue;
            };
            match ethean_network::prepare_blocks_by_root_for_roots(
                *peer,
                outcome.fetch_roots.clone(),
                &mut facade.requests,
            ) {
                Ok(Some(req)) => {
                    let n = outcome.fetch_roots.len();
                    facade.enqueue_blocks_outbounds(vec![req]);
                    match facade.flush_blocks_outbox() {
                        Ok(sent) => info!(
                            peer0 = peer[0],
                            parents = n,
                            sent,
                            "parent blocks-by-root flushed for orphan catch-up"
                        ),
                        Err(e) => info!(
                            peer0 = peer[0],
                            error = %e,
                            "parent blocks-by-root staged; flush deferred"
                        ),
                    }
                }
                Ok(None) => {}
                Err(e) => info!(peer0 = peer[0], error = %e, "parent fetch prepare failed"),
            }
        }
        Ok(())
    }
}
