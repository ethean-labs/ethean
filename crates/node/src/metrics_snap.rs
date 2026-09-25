//! Snapshot chain slot gauges into the process metrics registry.

use crate::api_view::fork_choice_view;
use crate::client::EtheanClient;
use crate::Result;
use ethean_genesis::SystemTimeSource;
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use ethean_rpc::{
    ApiSnapshot, DutiesView, DutyRow, FinalizedView, ForkChoiceStatsView, HeadView, SyncView,
};
use tracing::debug;

impl EtheanClient {
    /// Refresh Prometheus gauges used by the Lean Clients Grafana panels.
    pub(crate) fn refresh_slot_metrics(&mut self) -> Result<()> {
        let head_slot = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.slot.get())
            .or_else(|| self.owner.last_tick.map(|t| t.slot.get()))
            .unwrap_or(0);
        let (justified, finalized) = self
            .owner
            .head_state
            .as_ref()
            .map(|s| (s.latest_justified.slot.get(), s.latest_finalized.slot.get()))
            .unwrap_or((0, 0));
        let finalized_root = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.latest_finalized.root)
            .unwrap_or(HASH32_ZERO);
        let current = self
            .clock
            .slot_now(&SystemTimeSource)
            .map(|s| s.get())
            .unwrap_or(head_slot);
        let peers = {
            #[cfg(feature = "libp2p-quic")]
            {
                self.swarm
                    .as_ref()
                    .map(|s| s.connected_peer_count())
                    .unwrap_or(0)
            }
            #[cfg(not(feature = "libp2p-quic"))]
            {
                0u64
            }
        };
        let validators = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.validators.len() as u64)
            .unwrap_or(self.genesis.validators.len() as u64);
        let genesis_time = self
            .owner
            .head_state
            .as_ref()
            .map(|s| s.genesis_time())
            .unwrap_or_else(|| self.genesis.genesis_time());
        self.observability.record_slots(
            head_slot,
            justified,
            finalized,
            self.owner.safe_target_slot(),
            current,
            self.sync.lag(),
            peers,
        )?;
        self.observability
            .record_reorg_total(self.owner.reorg_total)?;
        if let Some(api) = self.api.as_ref() {
            let _ = self
                .observability
                .record_admin_event_backlog(api.events_pending() as u64);
        }
        let (peer_clients, mesh_clients) = {
            #[cfg(feature = "libp2p-quic")]
            {
                self.swarm
                    .as_ref()
                    .and_then(|s| s.quic.as_ref())
                    .map(|q| (q.peer_clients(), q.mesh_peer_clients()))
                    .unwrap_or_default()
            }
            #[cfg(not(feature = "libp2p-quic"))]
            {
                (Vec::new(), Vec::new())
            }
        };
        crate::lean_metrics::refresh_with_clients(
            &self.owner,
            current,
            peers,
            &peer_clients,
            &mesh_clients,
        );
        crate::lean_metrics::node_facts(&self.owner, env!("CARGO_PKG_VERSION"));
        self.observability.record_roles(
            validators,
            self.owner.is_aggregator,
            self.owner.local_finality,
            genesis_time,
            self.profile.seconds_per_slot,
        )?;
        self.observability.record_bootnodes(self.bootnode_count)?;
        self.observability.record_readiness_bits()?;
        self.observability.refresh_ready_gauge()?;
        #[cfg(feature = "libp2p-quic")]
        {
            let (found, missing, cache_slots) = self
                .swarm
                .as_ref()
                .map(|s| s.range_serve_stats())
                .unwrap_or((0, 0, 0));
            self.observability
                .record_range_serve(found, missing, cache_slots)?;
        }
        self.publish_lean_api(head_slot, finalized, finalized_root, peers);
        debug!(
            head_slot,
            justified,
            finalized,
            current,
            peers,
            validators,
            bootnodes = self.bootnode_count,
            "metrics slot snapshot"
        );
        Ok(())
    }

    fn publish_lean_api(
        &mut self,
        head_slot: u64,
        finalized_slot: u64,
        finalized_root: Hash32,
        peers: u64,
    ) {
        let Some(api) = self.api.as_ref() else {
            return;
        };
        api.set_ready(
            self.observability
                .ready_flag
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        // POST /lean/v0/admin/aggregator flips the RPC atomic; apply it onto the owner.
        self.owner.is_aggregator = api.is_aggregator();
        let peer_id = self.peer_id.clone();
        let _ = peers;
        let lag = self.sync.lag();
        let horizon = head_slot.saturating_add(if peers > 0 { lag.max(1) } else { 0 });
        let (justified_root, finalized_from_state) = self
            .owner
            .head_state
            .as_ref()
            .map(|s| (s.latest_justified.root, s.latest_finalized.root))
            .unwrap_or((HASH32_ZERO, finalized_root));
        let (blocks, pending_votes, known_votes, live) = match self.owner.fc.as_ref() {
            Some(fc) => (
                fc.blocks.len() as u64,
                fc.latest_new_attestations.len() as u64,
                fc.latest_known_attestations.len() as u64,
                true,
            ),
            None => (0, 0, 0, false),
        };
        let (tick_slot, tick_interval) = self
            .owner
            .last_tick
            .map(|t| (t.slot.get(), t.interval))
            .unwrap_or((head_slot, 0));
        let attester_loaded = self.owner.attester.is_some();
        let proposer_loaded = self.owner.proposer.is_some();
        let committees = self
            .owner
            .profile
            .as_ref()
            .map(|p| p.attestation_committee_count.max(1))
            .unwrap_or(1);
        let mut duty_rows = Vec::new();
        if attester_loaded && !self.owner.syncing {
            // Visibility matches try_local_attest: only the attestation interval.
            if tick_interval == crate::duty_attest::ATTESTATION_INTERVAL {
                for &idx in &self.owner.owned_validator_indices {
                    duty_rows.push(DutyRow {
                        validator_index: idx,
                        kind: "attestation",
                        slot: tick_slot,
                        // Same mapping as duty_attest publish path.
                        subnet: Some((idx % committees) as u16),
                    });
                }
            }
        }
        if proposer_loaded && !self.owner.syncing {
            // Visibility matches try_plan_proposal: Type-2 request window is
            // intervals 0..=2 (retry when the prover queue was busy).
            if tick_interval <= 2 {
                let n = self
                    .owner
                    .head_state
                    .as_ref()
                    .map(|s| s.validators.len() as u64)
                    .unwrap_or(0);
                if let Ok(proposer) =
                    ethean_transition::proposer_for_slot(Slot::new(tick_slot), n)
                {
                    let idx = proposer.get();
                    if self.owner.owned_validator_indices.contains(&idx) {
                        duty_rows.push(DutyRow {
                            validator_index: idx,
                            kind: "proposal",
                            slot: tick_slot,
                            subnet: None,
                        });
                    }
                }
            }
        }
        api.publish(ApiSnapshot {
            network: self.network_label.clone(),
            peer_id,
            head: HeadView {
                slot: Slot::new(head_slot),
                root: self.owner.head_root,
            },
            finalized: FinalizedView {
                slot: Slot::new(finalized_slot),
                root: finalized_root,
                trust_source: if self.owner.local_finality {
                    "local_finality".into()
                } else {
                    "checkpoint".into()
                },
            },
            sync: SyncView {
                syncing: lag > 0,
                head_slot: Slot::new(head_slot),
                peer_horizon_slot: Slot::new(horizon),
            },
            fork_choice: fork_choice_view(&self.owner),
            fork_choice_stats: ForkChoiceStatsView {
                live,
                head_root: self.owner.head_root,
                safe_target_root: self.owner.safe_target,
                safe_target_slot: self.owner.safe_target_slot(),
                justified_root,
                finalized_root: finalized_from_state,
                reorg_total: self.owner.reorg_total,
                blocks,
                pending_votes,
                known_votes,
            },
            duties: DutiesView {
                slot: tick_slot,
                interval: tick_interval,
                syncing: self.owner.syncing || lag > 0,
                is_aggregator: self.owner.is_aggregator,
                attester_loaded,
                proposer_loaded,
                attestation_committee_count: committees,
                owned_validator_indices: self.owner.owned_validator_indices.clone(),
                duties: duty_rows,
            },
            ready: api.is_ready(),
        });
    }
}
