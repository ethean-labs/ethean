//! Outbound attestation-subnet and aggregation-topic payloads.

use ethean_primitives::Hash32;

/// Cap on unpublished vote/aggregate payloads. Each aggregate is ~180 KB.
pub const MAX_PENDING_AGGREGATION_GOSSIP: usize = 32;

/// Gossip payload waiting for QuicSwarm publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregationGossip {
    /// Lean gossip topic string.
    pub topic: String,
    /// Uncompressed SSZ payload.
    pub payload: Vec<u8>,
    /// Attestation-data tree root.
    pub data_root: Hash32,
    /// Aggregate proof length (0 for an individual vote).
    pub proof_len: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain_owner::ChainOwner;

    fn item(i: u8) -> AggregationGossip {
        AggregationGossip {
            topic: "t".into(),
            payload: vec![i],
            data_root: [i; 32],
            proof_len: 0,
        }
    }

    #[test]
    fn pending_aggregation_gossip_drops_the_oldest() {
        let mut owner = ChainOwner::new(2);
        for i in 0..=MAX_PENDING_AGGREGATION_GOSSIP as u8 {
            owner.queue_aggregation_gossip(item(i));
        }
        assert_eq!(
            owner.pending_aggregation_gossip.len(),
            MAX_PENDING_AGGREGATION_GOSSIP
        );
        assert_eq!(owner.pending_aggregation_gossip[0].payload, vec![1]);
    }
}
