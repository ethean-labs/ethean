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
        self.apply_network_window(max_events, idle, None).await
    }

    /// [`Self::apply_network_budget`] that never waits past `deadline`.
    pub(crate) async fn apply_network_window(
        &mut self,
        max_events: u32,
        idle: std::time::Duration,
        deadline: Option<tokio::time::Instant>,
    ) -> Result<u32> {
        #[cfg(feature = "libp2p-quic")]
        {
            let budget = self.pump_network_window(max_events, idle, deadline).await?;
            let drained = budget.drained;
            self.drive_status_and_gossip(&budget)?;
            Ok(drained)
        }
        #[cfg(not(feature = "libp2p-quic"))]
        {
            let _ = (max_events, idle, deadline);
            let _ = self.status_sessions.pending_len();
            Ok(0)
        }
    }

    #[cfg(feature = "libp2p-quic")]
    fn refresh_local_status_bytes(&mut self) {
        let status = crate::status_handshake::build_local(&self.owner);
        if let Ok(bytes) = status.encode() {
            if let Some(facade) = self.swarm.as_mut() {
                let _ = facade.set_local_status_bytes(bytes);
            }
        }
        self.local_status = Some(status);
    }

    #[cfg(feature = "libp2p-quic")]
    fn flush_catchup_outbounds(&mut self, peer0: u8, out: crate::sync_catchup::CatchupOutbounds) {
        let Some(facade) = self.swarm.as_mut() else {
            return;
        };
        if let Some(blocks_req) = out.blocks_by_root {
            facade.enqueue_blocks_outbounds(vec![blocks_req]);
            match facade.flush_blocks_outbox() {
                Ok(sent) => info!(peer0, sent, "catch-up blocks-by-root flushed"),
                Err(e) => info!(
                    peer0,
                    error = %e,
                    "catch-up blocks-by-root staged; flush deferred"
                ),
            }
        }
        if let Some(range_req) = out.blocks_by_range {
            facade.enqueue_blocks_range_outbounds(vec![range_req]);
            match facade.flush_blocks_range_outbox() {
                Ok(sent) => info!(peer0, sent, "catch-up blocks-by-range flushed"),
                Err(e) => info!(
                    peer0,
                    error = %e,
                    "catch-up blocks-by-range staged; flush deferred"
                ),
            }
        }
    }

    #[cfg(feature = "libp2p-quic")]
    fn continue_sync_catchup(&mut self) {
        let local_head = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.slot.get())
            .unwrap_or(0);
        crate::sync_catchup::prune_caught_up(&mut self.sync_targets, local_head);
        if self.sync_targets.is_empty() {
            return;
        }
        let peer0 = self.sync_targets.first().map(|t| t.peer[0]).unwrap_or(0);
        let head_root = self.owner.head_root;
        let out = {
            let Some(facade) = self.swarm.as_mut() else {
                return;
            };
            match crate::sync_catchup::prepare_follow_up(
                &self.sync_targets,
                head_root,
                local_head,
                &mut facade.requests,
            ) {
                Ok(out) => out,
                Err(e) => {
                    info!(error = %e, "catch-up follow-up prepare failed");
                    return;
                }
            }
        };
        self.flush_catchup_outbounds(peer0, out);
    }

    #[cfg(feature = "libp2p-quic")]
    fn request_parent_blocks(
        &mut self,
        peer: ethean_primitives::Hash32,
        roots: Vec<ethean_primitives::Hash32>,
    ) {
        if roots.is_empty() {
            return;
        }
        let Some(facade) = self.swarm.as_mut() else {
            return;
        };
        let n = roots.len();
        match ethean_network::prepare_blocks_by_root_for_roots(peer, roots, &mut facade.requests) {
            Ok(Some(req)) => {
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

    #[cfg(feature = "libp2p-quic")]
    fn forget_disconnected_peers(&mut self, peers: &[ethean_primitives::Hash32]) {
        let mut dropped = 0usize;
        for peer in peers {
            self.status_sessions.on_peer_disconnected(peer);
            if crate::sync_catchup::forget_peer(&mut self.sync_targets, peer) {
                dropped += 1;
            }
        }
        if dropped == 0 {
            return;
        }
        let local_head = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.slot)
            .unwrap_or_default();
        crate::sync_catchup::apply_preferred_horizon(
            &self.sync_targets,
            &mut self.sync,
            local_head,
        );
        info!(
            dropped,
            targets = self.sync_targets.len(),
            lag = self.sync.lag(),
            "disconnected peers dropped from sync targets"
        );
    }

    #[cfg(feature = "libp2p-quic")]
    fn drive_status_and_gossip(
        &mut self,
        budget: &crate::swarm_pump::PumpBudgetResult,
    ) -> Result<()> {
        self.forget_disconnected_peers(&budget.disconnected_peers);
        if let Some(facade) = self.swarm.as_mut() {
            for peer in &budget.disconnected_peers {
                facade.requests.prune_peer(peer);
            }
            for (peer, _) in budget
                .status_responses
                .iter()
                .chain(&budget.blocks_by_root_responses)
                .chain(&budget.blocks_by_range_responses)
            {
                facade.requests.complete_oldest(peer);
            }
            let expired = facade
                .requests
                .expire(std::time::Instant::now(), ethean_network::REQUEST_EXPIRY);
            if expired > 0 {
                tracing::debug!(expired, "req/resp requests without a response dropped");
            }
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
                for peer in self.status_sessions.pending_peers() {
                    if !facade.is_peer_connected(&peer) {
                        self.status_sessions.on_peer_disconnected(&peer);
                    }
                }
                match ethean_network::prepare_status_outbounds(
                    &mut self.status_sessions,
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
            self.refresh_local_status_bytes();
            let imported =
                crate::serve_cache_seed::imported_gossip_blocks(&self.owner, &budget.accepted);
            if let Some(facade) = self.swarm.as_mut() {
                for (slot, root, bytes) in imported {
                    let _ = facade.put_block_at_slot(slot, root, bytes);
                }
            }
        }
        for (peer, parents) in
            crate::blocks_sync::buffer_gossip_orphans(&mut self.owner, &budget.accepted)
        {
            self.request_parent_blocks(peer, parents);
        }
        for (peer, payload) in &budget.status_responses {
            let handshake = {
                let Some(facade) = self.swarm.as_mut() else {
                    break;
                };
                crate::status_handshake::complete_status_handshake(
                    &mut self.status_sessions,
                    &mut self.sync,
                    &self.owner,
                    *peer,
                    payload,
                    &mut facade.requests,
                )
            };
            match handshake {
                Ok(out) => {
                    if let Some(remote) = out.remote {
                        crate::sync_catchup::remember_target(&mut self.sync_targets, *peer, remote);
                    }
                    let local_head = self
                        .owner
                        .head_state
                        .as_ref()
                        .map(|s| s.slot)
                        .unwrap_or_default();
                    crate::sync_catchup::apply_preferred_horizon(
                        &self.sync_targets,
                        &mut self.sync,
                        local_head,
                    );
                    info!(
                        peer0 = peer[0],
                        lag = self.sync.lag(),
                        targets = self.sync_targets.len(),
                        "Status tip remembered; majority catch-up follows"
                    );
                    self.continue_sync_catchup();
                }
                Err(e) => {
                    info!(peer0 = peer[0], error = %e, "Status handshake failed");
                }
            }
        }
        let mut ingested_blocks = false;
        let had_block_resp = !budget.blocks_by_root_responses.is_empty()
            || !budget.blocks_by_range_responses.is_empty();
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
                ingested_blocks = true;
                info!(
                    peer0 = peer[0],
                    n = outcome.events.len(),
                    orphans = self.owner.sync_orphans.len(),
                    "blocks sync response ingested"
                );
            }
            self.request_parent_blocks(*peer, outcome.fetch_roots);
        }
        if ingested_blocks {
            self.refresh_local_status_bytes();
            let local_head = self
                .owner
                .head_state
                .as_ref()
                .map(|s| s.slot)
                .unwrap_or_default();
            self.sync.observe_local(local_head);
        }
        // After a range/root response (even empty), keep fetching while tips remain ahead.
        // Status responses already call continue_sync_catchup after majority tip update.
        if had_block_resp {
            self.continue_sync_catchup();
        }
        Ok(())
    }
}
