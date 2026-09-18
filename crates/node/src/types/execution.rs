//! Execution layer types

use serde::{Deserialize, Serialize};

/// Execution payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub parent_hash: [u8; 32],
    pub fee_recipient: [u8; 20],
    pub state_root: [u8; 32],
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub block_hash: [u8; 32],
    pub transactions: Vec<Vec<u8>>,
}

/// Execution payload header
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayloadHeader {
    pub parent_hash: [u8; 32],
    pub fee_recipient: [u8; 20],
    pub state_root: [u8; 32],
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub block_hash: [u8; 32],
    pub transactions_root: [u8; 32],
}
