//! Outbound attestation-subnet and aggregation-topic payloads.

use ethean_primitives::Hash32;

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
