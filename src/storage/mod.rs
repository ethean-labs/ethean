//! Storage layer for Beam Chain client
//!
//! Provides database abstraction, state storage, block storage,
//! checkpoint management, and migration framework.

pub mod database;
pub mod state;
pub mod blocks;
pub mod checkpoints;
pub mod migrations;
pub mod memory_backend; // Test backend

pub use database::{Database, DatabaseConfig, DatabaseError};
pub use state::{StateStore, StateStorage};
pub use blocks::{BlockStore, BlockStorage};
pub use checkpoints::{CheckpointStore, CheckpointManager};
pub use migrations::{MigrationManager, Migration};
pub use memory_backend::InMemoryBackend;

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Enable state pruning
    pub enable_pruning: bool,
    /// Checkpoint interval (blocks)
    pub checkpoint_interval: u64,
    /// Archive mode (keep all historical data)
    pub archive_mode: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            enable_pruning: false,
            checkpoint_interval: 8192, // ~32 epochs at 4s slots
            archive_mode: false,
        }
    }
}

/// Main storage manager
pub struct StorageManager {
    database: Database,
    state_store: StateStore,
    block_store: BlockStore,
    checkpoint_manager: CheckpointManager,
    config: StorageConfig,
}

impl StorageManager {
    /// Create new storage manager
    pub fn new(config: StorageConfig) -> Result<Self, DatabaseError> {
        let database = Database::open(&config.database)?;
        let state_store = StateStore::new(database.clone());
        let block_store = BlockStore::new(database.clone());
        let checkpoint_manager = CheckpointManager::new(database.clone(), config.checkpoint_interval);
        
        Ok(Self {
            database,
            state_store,
            block_store,
            checkpoint_manager,
            config,
        })
    }

    /// Get state store
    pub fn state_store(&self) -> &StateStore {
        &self.state_store
    }

    /// Get block store
    pub fn block_store(&self) -> &BlockStore {
        &self.block_store
    }

    /// Get checkpoint manager
    pub fn checkpoint_manager(&self) -> &CheckpointManager {
        &self.checkpoint_manager
    }

    /// Close storage and flush all data
    pub fn close(self) -> Result<(), DatabaseError> {
        self.database.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_manager_creation() {
        let config = StorageConfig::default();
        let storage = StorageManager::new(config);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert!(!config.enable_pruning);
        assert_eq!(config.checkpoint_interval, 8192);
        assert!(!config.archive_mode);
    }
}