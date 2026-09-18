//! Block storage implementation
//!
//! Manages beacon block storage, retrieval, and block indexing.

use crate::types::block::{BeaconBlock, Root};
use crate::storage::database::{Database, DatabaseError};
use serde::{Serialize, Deserialize};

/// Block storage trait
pub trait BlockStorage: Send + Sync {
    /// Store beacon block
    fn store_block(&self, block_root: &Root, block: &BeaconBlock) -> Result<(), DatabaseError>;
    
    /// Get beacon block by root
    fn get_block(&self, block_root: &Root) -> Result<Option<BeaconBlock>, DatabaseError>;
    
    /// Get block by slot
    fn get_block_by_slot(&self, slot: u64) -> Result<Option<BeaconBlock>, DatabaseError>;
    
    /// Check if block exists
    fn has_block(&self, block_root: &Root) -> Result<bool, DatabaseError>;
    
    /// Delete block
    fn delete_block(&self, block_root: &Root) -> Result<(), DatabaseError>;
    
    /// Get blocks in range
    fn get_blocks_range(&self, start_slot: u64, end_slot: u64) -> Result<Vec<BeaconBlock>, DatabaseError>;
    
    /// Get latest block
    fn get_latest_block(&self) -> Result<Option<BeaconBlock>, DatabaseError>;
    
    /// Set head block
    fn set_head_block(&self, block_root: &Root) -> Result<(), DatabaseError>;
}

/// Block store implementation
#[derive(Clone)]
pub struct BlockStore {
    database: Database,
}

impl BlockStore {
    /// Create new block store
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Get block key for database
    fn block_key(block_root: &Root) -> Vec<u8> {
        let mut key = b"block:".to_vec();
        key.extend_from_slice(block_root);
        key
    }

    /// Get slot index key
    fn slot_index_key(slot: u64) -> Vec<u8> {
        let mut key = b"block_slot:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    /// Get head block key
    fn head_key() -> Vec<u8> {
        b"head_block".to_vec()
    }

    /// Get latest block key
    fn latest_key() -> Vec<u8> {
        b"latest_block".to_vec()
    }
}

impl BlockStorage for BlockStore {
    fn store_block(&self, block_root: &Root, block: &BeaconBlock) -> Result<(), DatabaseError> {
        let key = Self::block_key(block_root);
        self.database.put_typed(&key, block)?;
        
        // Index by slot for quick access
        let slot_key = Self::slot_index_key(block.slot);
        self.database.put(&slot_key, block_root)?;
        
        // Update latest block
        let latest_key = Self::latest_key();
        self.database.put(&latest_key, block_root)?;
        
        Ok(())
    }

    fn get_block(&self, block_root: &Root) -> Result<Option<BeaconBlock>, DatabaseError> {
        let key = Self::block_key(block_root);
        self.database.get_typed(&key)
    }

    fn get_block_by_slot(&self, slot: u64) -> Result<Option<BeaconBlock>, DatabaseError> {
        let slot_key = Self::slot_index_key(slot);
        
        if let Some(block_root_bytes) = self.database.get(&slot_key)? {
            if block_root_bytes.len() == 32 {
                let mut block_root = [0u8; 32];
                block_root.copy_from_slice(&block_root_bytes);
                return self.get_block(&block_root);
            }
        }
        
        Ok(None)
    }

    fn has_block(&self, block_root: &Root) -> Result<bool, DatabaseError> {
        let key = Self::block_key(block_root);
        self.database.exists(&key)
    }

    fn delete_block(&self, block_root: &Root) -> Result<(), DatabaseError> {
        let key = Self::block_key(block_root);
        self.database.delete(&key)?;
        
        // Note: We don't delete slot index to avoid expensive lookups
        // Multiple blocks can exist at same slot during reorgs
        
        Ok(())
    }

    fn get_blocks_range(&self, start_slot: u64, end_slot: u64) -> Result<Vec<BeaconBlock>, DatabaseError> {
        let mut blocks = Vec::new();
        
        for slot in start_slot..=end_slot {
            if let Some(block) = self.get_block_by_slot(slot)? {
                blocks.push(block);
            }
        }
        
        Ok(blocks)
    }

    fn get_latest_block(&self) -> Result<Option<BeaconBlock>, DatabaseError> {
        let key = Self::latest_key();
        
        if let Some(block_root_bytes) = self.database.get(&key)? {
            if block_root_bytes.len() == 32 {
                let mut block_root = [0u8; 32];
                block_root.copy_from_slice(&block_root_bytes);
                return self.get_block(&block_root);
            }
        }
        
        Ok(None)
    }

    fn set_head_block(&self, block_root: &Root) -> Result<(), DatabaseError> {
        let key = Self::head_key();
        self.database.put(&key, block_root)
    }
}

/// Block metadata for efficient querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Block slot
    pub slot: u64,
    /// Parent root
    pub parent_root: Root,
    /// State root
    pub state_root: Root,
    /// Proposer index
    pub proposer_index: u64,
    /// Block timestamp
    pub timestamp: u64,
}

/// Block index for fast queries
#[derive(Clone)]
pub struct BlockIndex {
    database: Database,
}

impl BlockIndex {
    /// Create new block index
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Index block metadata
    pub fn index_block(&self, block_root: &Root, metadata: &BlockMetadata) -> Result<(), DatabaseError> {
        let key = Self::metadata_key(block_root);
        self.database.put_typed(&key, metadata)?;
        
        // Index by proposer for validator queries
        let proposer_key = Self::proposer_index_key(metadata.proposer_index, metadata.slot);
        self.database.put(&proposer_key, block_root)?;
        
        Ok(())
    }

    /// Get block metadata
    pub fn get_metadata(&self, block_root: &Root) -> Result<Option<BlockMetadata>, DatabaseError> {
        let key = Self::metadata_key(block_root);
        self.database.get_typed(&key)
    }

    /// Get blocks by proposer
    pub fn get_blocks_by_proposer(&self, proposer_index: u64) -> Result<Vec<Root>, DatabaseError> {
        let prefix = format!("proposer:{}:", proposer_index).into_bytes();
        let keys = self.database.keys_with_prefix(&prefix)?;
        
        let mut blocks = Vec::new();
        for key in keys {
            if let Some(block_root_bytes) = self.database.get(&key)? {
                if block_root_bytes.len() == 32 {
                    let mut block_root = [0u8; 32];
                    block_root.copy_from_slice(&block_root_bytes);
                    blocks.push(block_root);
                }
            }
        }
        
        Ok(blocks)
    }

    /// Get metadata key
    fn metadata_key(block_root: &Root) -> Vec<u8> {
        let mut key = b"block_meta:".to_vec();
        key.extend_from_slice(block_root);
        key
    }

    /// Get proposer index key
    fn proposer_index_key(proposer_index: u64, slot: u64) -> Vec<u8> {
        format!("proposer:{}:{}", proposer_index, slot).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::DatabaseConfig;

    #[test]
    fn test_block_store_creation() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let store = BlockStore::new(db);
        
        // Test basic operations
        let block_root = [42u8; 32];
        let exists = store.has_block(&block_root).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_block_index() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let index = BlockIndex::new(db);
        
        let block_root = [42u8; 32];
        let metadata = BlockMetadata {
            slot: 123,
            parent_root: [1u8; 32],
            state_root: [2u8; 32],
            proposer_index: 456,
            timestamp: 1234567890,
        };
        
        let result = index.index_block(&block_root, &metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_key_generation() {
        let block_root = [42u8; 32];
        let key = BlockStore::block_key(&block_root);
        assert!(key.starts_with(b"block:"));
        assert_eq!(key.len(), 6 + 32);
        
        let slot_key = BlockStore::slot_index_key(123);
        assert!(slot_key.starts_with(b"block_slot:"));
        assert_eq!(slot_key.len(), 11 + 8);
    }

    #[test]
    fn test_blocks_range() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let store = BlockStore::new(db);
        
        let blocks = store.get_blocks_range(100, 110).unwrap();
        assert!(blocks.is_empty()); // No blocks stored yet
    }
}
