//! Block types - focused and minimal
//!
//! Single responsibility: block data structures only.

use serde::{Deserialize, Serialize};

pub type Slot = u64;
pub type BlockHash = [u8; 32];
pub type BlockBodyRoot = [u8; 32];
pub type Root = [u8; 32]; // Add Root type for storage layer

/// Minimal block header
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub proposer_index: u64,
    pub parent_root: BlockHash,
    pub state_root: [u8; 32],
    pub body_root: BlockBodyRoot,
}

/// Main block structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlock {
    pub slot: Slot,
    pub proposer_index: u64,
    pub parent_root: BlockHash,
    pub state_root: [u8; 32],
    pub body: BeaconBlockBody,
}

/// Block body - lightweight
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockBody {
    pub randao_reveal: Vec<u8>, // Use Vec instead of [u8; 96]
    pub graffiti: [u8; 32],
    pub attestations: Vec<super::attestation::Attestation>,
    pub execution_payload: Option<super::execution::ExecutionPayload>,
}

impl BeaconBlock {
    /// Get block hash
    pub fn hash(&self) -> BlockHash {
        use sha2::{Digest, Sha256};
        
        // Simple JSON serialization for now
        let encoded = serde_json::to_vec(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        hasher.finalize().into()
    }

    /// Get block header
    pub fn header(&self) -> BeaconBlockHeader {
        BeaconBlockHeader {
            slot: self.slot,
            proposer_index: self.proposer_index,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: self.body.hash(),
        }
    }
}

impl BeaconBlockBody {
    /// Get body hash
    pub fn hash(&self) -> BlockBodyRoot {
        use sha2::{Digest, Sha256};
        
        // Simple JSON serialization for now
        let encoded = serde_json::to_vec(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        hasher.finalize().into()
    }
}

impl BeaconBlockHeader {
    /// Get header hash
    pub fn hash(&self) -> BlockHash {
        use sha2::{Digest, Sha256};
        
        // Simple JSON serialization for now
        let encoded = serde_json::to_vec(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        hasher.finalize().into()
    }
}

impl Default for BeaconBlockHeader {
    fn default() -> Self {
        Self {
            slot: 0,
            proposer_index: 0,
            parent_root: [0u8; 32],
            state_root: [0u8; 32],
            body_root: [0u8; 32],
        }
    }
}

impl Default for BeaconBlock {
    fn default() -> Self {
        Self {
            slot: 0,
            proposer_index: 0,
            parent_root: [0u8; 32],
            state_root: [0u8; 32],
            body: BeaconBlockBody::default(),
        }
    }
}

impl Default for BeaconBlockBody {
    fn default() -> Self {
        Self {
            randao_reveal: vec![0u8; 96],
            graffiti: [0u8; 32],
            attestations: Vec::new(),
            execution_payload: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = BeaconBlock {
            slot: 100,
            proposer_index: 5,
            parent_root: [0u8; 32],
            state_root: [1u8; 32],
            body: BeaconBlockBody {
                randao_reveal: vec![0u8; 96],
                graffiti: [0u8; 32],
                attestations: vec![],
                execution_payload: None,
            },
        };

        assert_eq!(block.slot, 100);
        let hash = block.hash();
        assert_eq!(hash.len(), 32);
    }
}
