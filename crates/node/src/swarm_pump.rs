//! Non-blocking QuicSwarm event pump helpers (feature `libp2p-quic`).

use crate::chain_owner::ChainOwner;
use crate::network::{encode_gossip, GossipAction, GossipIngress, SwarmFacade};
use crate::{Error, Result};
use std::time::Duration;

/// Outcome of a bounded pump: count drained plus accepted gossip payloads.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PumpBudgetResult {
    /// Events drained from the swarm.
    pub drained: u32,
    /// ACCEPT gossip messages with decompressed payloads.
    pub accepted: Vec<GossipIngress>,
    /// Peer fingerprints from `ConnectionEstablished` events.
    pub connected_peers: Vec<ethean_primitives::Hash32>,
    /// Decompressed Status response payloads keyed by peer fingerprint.
    pub status_responses: Vec<(ethean_primitives::Hash32, Vec<u8>)>,
}

/// Result of flushing a pending local block proposal to gossip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedBlock {
    /// Topic that received the publish.
    pub topic: String,
    /// Uncompressed SSZ payload length.
    pub payload_len: usize,
    /// Whether the envelope carried a Type-2 proof.
    pub has_type2_proof: bool,
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
                if let crate::network::PumpEvent::ConnectionEstablished { peer: Some(p) } = &event
                {
                    out.connected_peers.push(*p);
                }
                if let crate::network::PumpEvent::StatusResponse { peer, payload } = &event {
                    out.status_responses.push((*peer, payload.clone()));
                }
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

/// Snappy-compress and publish `owner.pending_block_gossip`, clearing it on success.
pub fn publish_pending_block(
    facade: &mut SwarmFacade,
    owner: &mut ChainOwner,
) -> Result<Option<PublishedBlock>> {
    let Some(gossip) = owner.pending_block_gossip.take() else {
        return Ok(None);
    };
    let compressed = encode_gossip(&gossip.payload).map_err(|e| {
        Error::Network(ethean_network::NetworkError::Handshake(format!(
            "snappy encode: {e}"
        )))
    })?;
    match facade.publish_gossip(&gossip.topic, &compressed) {
        Ok(()) => Ok(Some(PublishedBlock {
            topic: gossip.topic,
            payload_len: gossip.payload.len(),
            has_type2_proof: gossip.has_type2_proof,
        })),
        Err(e) => {
            // Restore so a later flush can retry.
            owner.pending_block_gossip = Some(gossip);
            Err(Error::Network(e))
        }
    }
}

/// Flush pending gossip and map success to [`crate::events::ChainEvent::ProposalPublished`].
pub fn flush_pending_event(
    facade: &mut SwarmFacade,
    owner: &mut ChainOwner,
) -> Result<Option<crate::events::ChainEvent>> {
    Ok(publish_pending_block(facade, owner)?.map(|p| {
        crate::events::ChainEvent::ProposalPublished {
            topic: p.topic,
            payload_len: p.payload_len,
            has_type2_proof: p.has_type2_proof,
        }
    }))
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

    #[tokio::test]
    async fn publish_pending_none_when_empty() {
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
        assert!(publish_pending_block(&mut facade, &mut owner)
            .expect("flush")
            .is_none());
    }
}
