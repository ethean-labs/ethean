//! Flush pending Type-1 aggregation gossip on QuicSwarm.

use crate::aggregation_gossip::AggregationGossip;
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::network::{encode_gossip, SwarmFacade};
use crate::{Error, Result};

/// Snappy-compress and publish one queued Type-1 aggregate; restores on hard failure.
pub fn publish_one_aggregation(
    facade: &mut SwarmFacade,
    owner: &mut ChainOwner,
) -> Result<Option<AggregationGossip>> {
    if owner.pending_aggregation_gossip.is_empty() {
        return Ok(None);
    }
    let gossip = owner.pending_aggregation_gossip.remove(0);
    let compressed = encode_gossip(&gossip.payload).map_err(|e| {
        Error::Network(ethean_network::NetworkError::Handshake(format!(
            "snappy encode: {e}"
        )))
    })?;
    match facade.publish_gossip(&gossip.topic, &compressed) {
        Ok(()) => Ok(Some(gossip)),
        Err(e) => {
            let soft = owner.local_finality
                && matches!(&e, ethean_network::NetworkError::Handshake(msg)
                    if msg.contains("InsufficientPeers"));
            if soft {
                tracing::debug!(
                    error = %e,
                    topic = %gossip.topic,
                    "aggregation gossip publish skipped (no peers)"
                );
                return Ok(Some(gossip));
            }
            owner.pending_aggregation_gossip.insert(0, gossip);
            Err(Error::Network(e))
        }
    }
}

/// Drain all pending aggregation gossip into [`ChainEvent::AggregationPublished`].
pub fn flush_pending_aggregations(
    facade: &mut SwarmFacade,
    owner: &mut ChainOwner,
) -> Result<Vec<ChainEvent>> {
    let mut out = Vec::new();
    while !owner.pending_aggregation_gossip.is_empty() {
        match publish_one_aggregation(facade, owner)? {
            Some(g) => out.push(ChainEvent::AggregationPublished {
                topic: g.topic,
                data_root: g.data_root,
                payload_len: g.payload.len(),
            }),
            None => break,
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::TransportConfig;

    #[tokio::test]
    async fn flush_empty_is_ok() {
        let mut facade = SwarmFacade::default();
        facade
            .bind_quic_swarm_for_fork(
                &TransportConfig {
                    listen_port: 0,
                    idle_timeout_ms: 1_000,
                },
                "lstar",
            )
            .await
            .expect("bind");
        let mut owner = ChainOwner::new(2);
        assert!(flush_pending_aggregations(&mut facade, &mut owner)
            .expect("flush")
            .is_empty());
    }
}
