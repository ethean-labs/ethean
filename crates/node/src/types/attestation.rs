//! Attestation types - minimal and focused

use serde::{Deserialize, Serialize};

/// What validators attest to
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationData {
    pub slot: super::block::Slot,
    pub index: u64,
    pub beacon_block_root: super::block::BlockHash,
    pub source: super::checkpoint::Checkpoint,
    pub target: super::checkpoint::Checkpoint,
}

/// Single attestation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub aggregation_bits: Vec<bool>,
    pub data: AttestationData,
    pub signature: Vec<u8>, // Use Vec instead of [u8; 96]
}
