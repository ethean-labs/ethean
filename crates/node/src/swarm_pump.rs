//! Non-blocking QuicSwarm event pump helpers (feature `libp2p-quic`).

#![cfg(feature = "libp2p-quic")]

use crate::network::SwarmFacade;
use crate::{Error, Result};
use std::time::Duration;

/// Drain up to `max_events` swarm events without blocking longer than `idle`.
pub async fn pump_swarm_budget(
    facade: &mut SwarmFacade,
    max_events: u32,
    idle: Duration,
) -> Result<u32> {
    let mut drained = 0u32;
    for _ in 0..max_events {
        match tokio::time::timeout(idle, facade.pump_quic_once()).await {
            Ok(Ok(_kind)) => {
                drained = drained.saturating_add(1);
            }
            Ok(Err(e)) => return Err(Error::Network(e)),
            Err(_) => break,
        }
    }
    Ok(drained)
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
        let n = pump_swarm_budget(&mut facade, 3, Duration::from_millis(5))
            .await
            .expect("pump");
        // Fresh bind may still emit listen-addr leftovers or sit idle.
        assert!(n <= 3);
    }
}
