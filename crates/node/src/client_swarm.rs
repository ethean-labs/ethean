//! QuicSwarm accessors and gossip flush for [`EtheanClient`].

use crate::client::EtheanClient;
use crate::Result;

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
