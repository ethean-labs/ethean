//! Checkpoint management system
//!
//! Handles beacon state checkpoints for fast sync and state recovery.

use crate::types::state::BeaconState;
use crate::types::block::Root;
use crate::storage::database::{Database, DatabaseError};
use serde::{Serialize, Deserialize};

/// Checkpoint data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint epoch
    pub epoch: u64,
    /// State root at checkpoint
    pub state_root: Root,
    /// Block root at checkpoint
    pub block_root: Root,
    /// Checkpoint timestamp
    pub timestamp: u64,
    /// Checkpoint size in bytes
    pub size: u64,
}

/// Checkpoint storage trait
pub trait CheckpointStore: Send + Sync {
    /// Store checkpoint
    fn store_checkpoint(&self, checkpoint: &Checkpoint, state: &BeaconState) -> Result<(), DatabaseError>;
    
    /// Get checkpoint by epoch
    fn get_checkpoint(&self, epoch: u64) -> Result<Option<Checkpoint>, DatabaseError>;
    
    /// Get checkpoint state
    fn get_checkpoint_state(&self, epoch: u64) -> Result<Option<BeaconState>, DatabaseError>;
    
    /// List available checkpoints
    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, DatabaseError>;
    
    /// Delete checkpoint
    fn delete_checkpoint(&self, epoch: u64) -> Result<(), DatabaseError>;
    
    /// Get latest checkpoint
    fn get_latest_checkpoint(&self) -> Result<Option<Checkpoint>, DatabaseError>;
    
    /// Prune old checkpoints
    fn prune_checkpoints(&self, keep_count: usize) -> Result<Vec<u64>, DatabaseError>;
}

/// Checkpoint manager implementation
#[derive(Clone)]
pub struct CheckpointManager {
    database: Database,
    interval: u64, // Checkpoint every N blocks
}

impl CheckpointManager {
    /// Create new checkpoint manager
    pub fn new(database: Database, interval: u64) -> Self {
        Self { database, interval }
    }

    /// Get checkpoint interval
    pub fn interval(&self) -> u64 {
        self.interval
    }

    /// Check if slot should create checkpoint
    pub fn should_checkpoint(&self, slot: u64) -> bool {
        slot % self.interval == 0
    }

    /// Get checkpoint key for database
    fn checkpoint_key(epoch: u64) -> Vec<u8> {
        let mut key = b"checkpoint:".to_vec();
        key.extend_from_slice(&epoch.to_be_bytes());
        key
    }

    /// Get checkpoint state key
    fn checkpoint_state_key(epoch: u64) -> Vec<u8> {
        let mut key = b"checkpoint_state:".to_vec();
        key.extend_from_slice(&epoch.to_be_bytes());
        key
    }

    /// Get latest checkpoint key
    fn latest_checkpoint_key() -> Vec<u8> {
        b"latest_checkpoint".to_vec()
    }
}

impl CheckpointStore for CheckpointManager {
    fn store_checkpoint(&self, checkpoint: &Checkpoint, state: &BeaconState) -> Result<(), DatabaseError> {
        // Store checkpoint metadata
        let meta_key = Self::checkpoint_key(checkpoint.epoch);
        self.database.put_typed(&meta_key, checkpoint)?;
        
        // Store checkpoint state
        let state_key = Self::checkpoint_state_key(checkpoint.epoch);
        self.database.put_typed(&state_key, state)?;
        
        // Update latest checkpoint
        let latest_key = Self::latest_checkpoint_key();
        self.database.put(&latest_key, &checkpoint.epoch.to_be_bytes())?;
        
        Ok(())
    }

    fn get_checkpoint(&self, epoch: u64) -> Result<Option<Checkpoint>, DatabaseError> {
        let key = Self::checkpoint_key(epoch);
        self.database.get_typed(&key)
    }

    fn get_checkpoint_state(&self, epoch: u64) -> Result<Option<BeaconState>, DatabaseError> {
        let key = Self::checkpoint_state_key(epoch);
        self.database.get_typed(&key)
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, DatabaseError> {
        let prefix = b"checkpoint:";
        let keys = self.database.keys_with_prefix(prefix)?;
        
        let mut checkpoints = Vec::new();
        for key in keys {
            if let Some(data) = self.database.get(&key)? {
                if let Ok(checkpoint) = serde_json::from_slice::<Checkpoint>(&data) {
                    checkpoints.push(checkpoint);
                }
            }
        }
        
        // Sort by epoch
        checkpoints.sort_by_key(|c| c.epoch);
        Ok(checkpoints)
    }

    fn delete_checkpoint(&self, epoch: u64) -> Result<(), DatabaseError> {
        let meta_key = Self::checkpoint_key(epoch);
        let state_key = Self::checkpoint_state_key(epoch);
        
        self.database.delete(&meta_key)?;
        self.database.delete(&state_key)?;
        
        Ok(())
    }

    fn get_latest_checkpoint(&self) -> Result<Option<Checkpoint>, DatabaseError> {
        let latest_key = Self::latest_checkpoint_key();
        
        if let Some(epoch_bytes) = self.database.get(&latest_key)? {
            if epoch_bytes.len() == 8 {
                let epoch = u64::from_be_bytes([
                    epoch_bytes[0], epoch_bytes[1], epoch_bytes[2], epoch_bytes[3],
                    epoch_bytes[4], epoch_bytes[5], epoch_bytes[6], epoch_bytes[7],
                ]);
                return self.get_checkpoint(epoch);
            }
        }
        
        Ok(None)
    }

    fn prune_checkpoints(&self, keep_count: usize) -> Result<Vec<u64>, DatabaseError> {
        let checkpoints = self.list_checkpoints()?;
        
        if checkpoints.len() <= keep_count {
            return Ok(Vec::new());
        }
        
        let to_delete = checkpoints.len() - keep_count;
        let mut deleted = Vec::new();
        
        for checkpoint in checkpoints.iter().take(to_delete) {
            self.delete_checkpoint(checkpoint.epoch)?;
            deleted.push(checkpoint.epoch);
        }
        
        Ok(deleted)
    }
}

/// Checkpoint creation helper
pub struct CheckpointCreator {
    manager: CheckpointManager,
}

impl CheckpointCreator {
    /// Create new checkpoint creator
    pub fn new(manager: CheckpointManager) -> Self {
        Self { manager }
    }

    /// Create checkpoint from state
    pub fn create_checkpoint(
        &self,
        epoch: u64,
        state: &BeaconState,
        block_root: &Root,
    ) -> Result<Checkpoint, DatabaseError> {
        let state_root = [0u8; 32]; // TODO: Calculate actual state root
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Estimate state size (simplified)
        let size = serde_json::to_vec(state)
            .map(|v| v.len() as u64)
            .unwrap_or(0);
        
        let checkpoint = Checkpoint {
            epoch,
            state_root,
            block_root: *block_root,
            timestamp,
            size,
        };
        
        self.manager.store_checkpoint(&checkpoint, state)?;
        Ok(checkpoint)
    }

    /// Auto-create checkpoint if needed
    pub fn maybe_create_checkpoint(
        &self,
        slot: u64,
        state: &BeaconState,
        block_root: &Root,
    ) -> Result<Option<Checkpoint>, DatabaseError> {
        if self.manager.should_checkpoint(slot) {
            let epoch = slot / 32; // Assuming 32 slots per epoch
            let checkpoint = self.create_checkpoint(epoch, state, block_root)?;
            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::DatabaseConfig;

    #[test]
    fn test_checkpoint_manager_creation() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let manager = CheckpointManager::new(db, 1000);
        
        assert_eq!(manager.interval(), 1000);
        assert!(manager.should_checkpoint(1000));
        assert!(!manager.should_checkpoint(999));
        assert!(manager.should_checkpoint(2000));
    }

    #[test]
    fn test_checkpoint_creation() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let manager = CheckpointManager::new(db.clone(), 1000);
        let creator = CheckpointCreator::new(manager);
        
        let state = BeaconState::default();
        let block_root = [42u8; 32];
        
        let checkpoint = creator.create_checkpoint(10, &state, &block_root);
        assert!(checkpoint.is_ok());
        
        let cp = checkpoint.unwrap();
        assert_eq!(cp.epoch, 10);
        assert_eq!(cp.block_root, block_root);
        assert!(cp.size > 0);
    }

    #[test]
    fn test_checkpoint_maybe_create() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let manager = CheckpointManager::new(db.clone(), 32);
        let creator = CheckpointCreator::new(manager);
        
        let state = BeaconState::default();
        let block_root = [42u8; 32];
        
        // Should create checkpoint
        let result = creator.maybe_create_checkpoint(64, &state, &block_root);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        
        // Should not create checkpoint
        let result = creator.maybe_create_checkpoint(65, &state, &block_root);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_checkpoint_keys() {
        let key = CheckpointManager::checkpoint_key(123);
        assert!(key.starts_with(b"checkpoint:"));
        assert_eq!(key.len(), 11 + 8);
        
        let state_key = CheckpointManager::checkpoint_state_key(123);
        assert!(state_key.starts_with(b"checkpoint_state:"));
        assert_eq!(state_key.len(), 17 + 8);
    }
}
