//! Snapshot chain slot gauges into the process metrics registry.

use crate::client::EtheanClient;
use crate::Result;
use ethean_genesis::SystemTimeSource;
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use ethean_rpc::{ApiSnapshot, FinalizedView, HeadView, SyncView};
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
        crate::lean_metrics::refresh(&self.owner, current, peers);
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
        &self,
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
        let peer_id = String::new();
        let _ = peers;
        let lag = self.sync.lag();
        let horizon = head_slot.saturating_add(if peers > 0 { lag.max(1) } else { 0 });
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
        });
    }
}
