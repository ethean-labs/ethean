//! Block storage — SSZ bytes.

use crate::storage::database::{Database, DatabaseError};
use ethean_types::{Block, Root};

/// Block storage trait
pub trait BlockStorage: Send + Sync {
    fn store_block(&self, block_root: &Root, block: &Block) -> Result<(), DatabaseError>;
    fn get_block(&self, block_root: &Root) -> Result<Option<Block>, DatabaseError>;
    fn get_block_by_slot(&self, slot: u64) -> Result<Option<Block>, DatabaseError>;
    fn has_block(&self, block_root: &Root) -> Result<bool, DatabaseError>;
    fn delete_block(&self, block_root: &Root) -> Result<(), DatabaseError>;
    fn get_blocks_range(&self, start_slot: u64, end_slot: u64) -> Result<Vec<Block>, DatabaseError>;
    fn get_latest_block(&self) -> Result<Option<Block>, DatabaseError>;
}

/// Block store backed by the node database.
#[derive(Clone)]
pub struct BlockStore {
    database: Database,
}

impl BlockStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    fn block_key(block_root: &Root) -> Vec<u8> {
        let mut key = b"block:".to_vec();
        key.extend_from_slice(block_root);
        key
    }

    fn slot_index_key(slot: u64) -> Vec<u8> {
        let mut key = b"block_slot:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    fn latest_key() -> Vec<u8> {
        b"latest_block".to_vec()
    }
}

impl BlockStorage for BlockStore {
    fn store_block(&self, block_root: &Root, block: &Block) -> Result<(), DatabaseError> {
        let key = Self::block_key(block_root);
        let bytes = block
            .ssz_encode()
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        self.database.put(&key, &bytes)?;
        let slot_key = Self::slot_index_key(block.slot.get());
        self.database.put(&slot_key, block_root)?;
        self.database.put(&Self::latest_key(), block_root)?;
        Ok(())
    }

    fn get_block(&self, block_root: &Root) -> Result<Option<Block>, DatabaseError> {
        let key = Self::block_key(block_root);
        match self.database.get(&key)? {
            None => Ok(None),
            Some(bytes) => Block::ssz_decode(&bytes)
                .map(Some)
                .map_err(|e| DatabaseError::SerializationError(e.to_string())),
        }
    }

    fn get_block_by_slot(&self, slot: u64) -> Result<Option<Block>, DatabaseError> {
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
        self.database.exists(&Self::block_key(block_root))
    }

    fn delete_block(&self, block_root: &Root) -> Result<(), DatabaseError> {
        self.database.delete(&Self::block_key(block_root))
    }

    fn get_blocks_range(
        &self,
        start_slot: u64,
        end_slot: u64,
    ) -> Result<Vec<Block>, DatabaseError> {
        let mut out = Vec::new();
        for slot in start_slot..=end_slot {
            if let Some(block) = self.get_block_by_slot(slot)? {
                out.push(block);
            }
        }
        Ok(out)
    }

    fn get_latest_block(&self) -> Result<Option<Block>, DatabaseError> {
        if let Some(bytes) = self.database.get(&Self::latest_key())? {
            if bytes.len() == 32 {
                let mut root = [0u8; 32];
                root.copy_from_slice(&bytes);
                return self.get_block(&root);
            }
        }
        Ok(None)
    }
}
