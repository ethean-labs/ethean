//! Lean gossip topic strings (Phase 10 shapes; never Beacon `/eth2/`).

use ethean_network_wire::{
    fork_segment_from_name, topic_aggregation, topic_attestation, topic_block,
};

use crate::error::{NetworkError, Result};

/// Canonical Lean gossip topics for a fork name (e.g. `lstar`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanGossipTopics {
    /// `/leanconsensus/{fork}/block/ssz_snappy`
    pub block: String,
    /// `/leanconsensus/{fork}/aggregation/ssz_snappy`
    pub aggregation: String,
    /// Attestation subnet 0 (smoke / single-subnet mesh).
    pub attestation_0: String,
}

impl LeanGossipTopics {
    /// Build topics from a profile fork name.
    pub fn from_fork_name(fork_name: &str) -> Result<Self> {
        let fork = fork_segment_from_name(fork_name)
            .map_err(|e| NetworkError::Handshake(e.to_string()))?;
        Ok(Self {
            block: topic_block(&fork).map_err(|e| NetworkError::Handshake(e.to_string()))?,
            aggregation: topic_aggregation(&fork)
                .map_err(|e| NetworkError::Handshake(e.to_string()))?,
            attestation_0: topic_attestation(&fork, 0)
                .map_err(|e| NetworkError::Handshake(e.to_string()))?,
        })
    }

    /// All mesh topics in subscription order.
    pub fn as_slice(&self) -> [&str; 3] {
        [&self.block, &self.aggregation, &self.attestation_0]
    }

    /// Refuse Beacon topic reuse.
    pub fn reject_if_eth2(topic: &str) -> Result<()> {
        if topic.contains("/eth2/") {
            return Err(NetworkError::Handshake(
                "Beacon /eth2/ gossip topics refused".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lstar_topics_are_lean() {
        let t = LeanGossipTopics::from_fork_name("lstar").unwrap();
        assert!(t.block.contains("/leanconsensus/"));
        assert!(t.block.ends_with("/block/ssz_snappy"));
        assert!(!t.block.contains("12345678"));
        assert!(LeanGossipTopics::reject_if_eth2("/eth2/beacon_block").is_err());
    }
}
