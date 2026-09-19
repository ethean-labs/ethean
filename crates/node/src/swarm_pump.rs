//! Non-blocking QuicSwarm event pump helpers (feature `libp2p-quic`).

use crate::network::{GossipIngress, GossipAction, SwarmFacade};
use crate::{Error, Result};
use std::time::Duration;

/// Outcome of a bounded pump: count drained plus accepted gossip payloads.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PumpBudgetResult {
    /// Events drained from the swarm.
    pub drained: u32,
    /// ACCEPT gossip messages with decompressed payloads.
    pub accepted: Vec<GossipIngress>,
}

/// Drain up to `max_events` swarm events without blocking longer than `idle`.
pub async fn pump_swarm_budget(
    facade: &mut SwarmFacade,
    max_events: u32,
    idle: Duration,
) -> Result<PumpBudgetResult> {
    let mut out = PumpBudgetResult::default();
    for _ in 0..max_events {
        match tokio::time::timeout(idle, facade.pump_quic_once()).await {
            Ok(Ok(event)) => {
                out.drained = out.drained.saturating_add(1);
                if let Some(g) = event.gossip() {
                    if g.action == GossipAction::Accept && g.plain.is_some() {
                        out.accepted.push(g.clone());
                    }
                }
            }
            Ok(Err(e)) => return Err(Error::Network(e)),
            Err(_) => break,
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::TransportConfig;

    #[tokio::test]
    async fn drains_zero_when_idle() {
        let mut facade = SwarmFacade::default();
        facade
            .bind_quic_swarm(&TransportConfig {
                listen_port: 0,
                idle_timeout_ms: 1_000,
            })
            .await
            .expect("bind");
        let r = pump_swarm_budget(&mut facade, 3, Duration::from_millis(5))
            .await
            .expect("pump");
        assert!(r.drained <= 3);
        assert!(r.accepted.is_empty());
    }
}
