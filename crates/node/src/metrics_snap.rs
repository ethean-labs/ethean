//! Snapshot chain slot gauges into the process metrics registry.

use crate::client::EtheanClient;
use crate::Result;
use ethean_genesis::{SystemTimeSource, TimeSource};
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
            .map(|s| {
                (
                    s.latest_justified.slot.get(),
                    s.latest_finalized.slot.get(),
                )
            })
            .unwrap_or((0, 0));
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
                    .map(|s| s.peers.len() as u64)
                    .unwrap_or(0)
            }
            #[cfg(not(feature = "libp2p-quic"))]
            {
                0u64
            }
        };
        self.observability.record_slots(
            head_slot,
            justified,
            finalized,
            current,
            self.sync.lag(),
            peers,
        )?;
        self.observability.refresh_ready_gauge()?;
        debug!(
            head_slot,
            justified,
            finalized,
            current,
            peers,
            "metrics slot snapshot"
        );
        Ok(())
    }
}
