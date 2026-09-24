//! Lean gossip topic strings (Phase 10 shapes; never Beacon `/eth2/`).

use ethean_network_wire::{
    fork_segment_from_name, topic_aggregation, topic_attestation, topic_block,
};

use crate::error::{NetworkError, Result};

/// Default attestation subnet count: leanSpec `ATTESTATION_COMMITTEE_COUNT` (lstar = 1).
pub const SMOKE_ATTESTATION_SUBNETS: u16 = 1;

/// Canonical Lean gossip topics for a fork name (e.g. `lstar`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanGossipTopics {
    /// `/leanconsensus/{fork}/block/ssz_snappy`
    pub block: String,
    /// `/leanconsensus/{fork}/aggregation/ssz_snappy`
    pub aggregation: String,
    /// Attestation subnet topics `attestation_0` .. `attestation_{N-1}`.
    pub attestations: Vec<String>,
}

impl LeanGossipTopics {
    /// Build topics from a profile fork name (lstar `GOSSIP_DIGEST` by default).
    pub fn from_fork_name(fork_name: &str) -> Result<Self> {
        let fork = fork_segment_from_name(fork_name)
            .map_err(|e| NetworkError::Handshake(e.to_string()))?;
        Self::from_fork_segment(&fork)
    }

    /// Build topics from an already-resolved 8-hex fork segment (operator pin).
    pub fn from_fork_segment(fork_segment: &str) -> Result<Self> {
        Self::from_fork_segment_subnets(fork_segment, SMOKE_ATTESTATION_SUBNETS)
    }

    /// Build topics with an explicit attestation subnet count.
    pub fn from_fork_segment_subnets(fork_segment: &str, subnet_count: u16) -> Result<Self> {
        let count = subnet_count.max(1);
        let mut attestations = Vec::with_capacity(count as usize);
        for subnet in 0..count {
            attestations.push(
                topic_attestation(fork_segment, subnet)
                    .map_err(|e| NetworkError::Handshake(e.to_string()))?,
            );
        }
        Ok(Self {
            block: topic_block(fork_segment).map_err(|e| NetworkError::Handshake(e.to_string()))?,
            aggregation: topic_aggregation(fork_segment)
                .map_err(|e| NetworkError::Handshake(e.to_string()))?,
            attestations,
        })
    }

    /// Subnet 0 topic (compat for single-subnet callers).
    pub fn attestation_0(&self) -> &str {
        self.attestations
            .first()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// All mesh topics in subscription order.
    pub fn as_slice(&self) -> Vec<&str> {
        let mut out = Vec::with_capacity(2 + self.attestations.len());
        out.push(self.block.as_str());
        out.push(self.aggregation.as_str());
        out.extend(self.attestations.iter().map(|s| s.as_str()));
        out
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
        assert!(t.block.contains("12345678"));
        assert_eq!(t.attestations.len(), SMOKE_ATTESTATION_SUBNETS as usize);
        assert!(t.attestation_0().contains("/attestation_0/"));
        assert!(LeanGossipTopics::reject_if_eth2("/eth2/beacon_block").is_err());
    }

    #[test]
    fn segment_override_topics() {
        let t = LeanGossipTopics::from_fork_segment("aabbccdd").unwrap();
        assert!(t.block.contains("/leanconsensus/aabbccdd/"));
        assert_eq!(t.as_slice().len(), 3);
        assert!(t.attestations.iter().any(|s| s.contains("attestation_0")));
        let four = LeanGossipTopics::from_fork_segment_subnets("aabbccdd", 4).unwrap();
        assert!(four.attestations.iter().any(|s| s.contains("attestation_3")));
    }
}
