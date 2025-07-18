//! State storage implementation
//!
//! Manages beacon state storage, retrieval, and historical state access.

use crate::types::state::BeaconState;
use crate::types::block::Root;
use crate::storage::database::{Database, DatabaseError};
use std::collections::HashMap;

/// State storage trait
pub trait StateStorage: Send + Sync {
    /// Store beacon state
    fn store_state(&self, state_root: &Root, state: &BeaconState) -> Result<(), DatabaseError>;
    
    /// Get beacon state by root
    fn get_state(&self, state_root: &Root) -> Result<Option<BeaconState>, DatabaseError>;
    
    /// Get state by slot
    fn get_state_by_slot(&self, slot: u64) -> Result<Option<BeaconState>, DatabaseError>;
    
    /// Check if state exists
    fn has_state(&self, state_root: &Root) -> Result<bool, DatabaseError>;
    
    /// Delete state
    fn delete_state(&self, state_root: &Root) -> Result<(), DatabaseError>;
    
    /// Get latest finalized state
    fn get_finalized_state(&self) -> Result<Option<BeaconState>, DatabaseError>;
    
    /// Set finalized state
    fn set_finalized_state(&self, state_root: &Root) -> Result<(), DatabaseError>;
}

/// State store implementation
#[derive(Clone)]
pub struct StateStore {
    database: Database,
}

impl StateStore {
    /// Create new state store
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Get state key for database
    fn state_key(state_root: &Root) -> Vec<u8> {
        let mut key = b"state:".to_vec();
        key.extend_from_slice(state_root);
        key
    }

    /// Get slot index key
    fn slot_index_key(slot: u64) -> Vec<u8> {
        let mut key = b"slot_index:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    /// Get finalized state key
    fn finalized_key() -> Vec<u8> {
        b"finalized_state".to_vec()
    }
}

impl StateStorage for StateStore {
    fn store_state(&self, state_root: &Root, state: &BeaconState) -> Result<(), DatabaseError> {
        let key = Self::state_key(state_root);
        self.database.put_typed(&key, state)?;
        
        // Also index by slot for quick access
        let slot_key = Self::slot_index_key(state.slot);
        self.database.put(&slot_key, state_root)?;
        
        Ok(())
    }

    fn get_state(&self, state_root: &Root) -> Result<Option<BeaconState>, DatabaseError> {
        let key = Self::state_key(state_root);
        self.database.get_typed(&key)
    }

    fn get_state_by_slot(&self, slot: u64) -> Result<Option<BeaconState>, DatabaseError> {
        let slot_key = Self::slot_index_key(slot);
        
        if let Some(state_root_bytes) = self.database.get(&slot_key)? {
            if state_root_bytes.len() == 32 {
                let mut state_root = [0u8; 32];
                state_root.copy_from_slice(&state_root_bytes);
                return self.get_state(&state_root);
            }
        }
        
        Ok(None)
    }

    fn has_state(&self, state_root: &Root) -> Result<bool, DatabaseError> {
        let key = Self::state_key(state_root);
        self.database.exists(&key)
    }

    fn delete_state(&self, state_root: &Root) -> Result<(), DatabaseError> {
        let key = Self::state_key(state_root);
        self.database.delete(&key)?;
        
        // Note: We don't delete slot index here as it might be expensive
        // to find and multiple states could have same slot during reorgs
        
        Ok(())
    }

    fn get_finalized_state(&self) -> Result<Option<BeaconState>, DatabaseError> {
        let key = Self::finalized_key();
        
        if let Some(state_root_bytes) = self.database.get(&key)? {
            if state_root_bytes.len() == 32 {
                let mut state_root = [0u8; 32];
                state_root.copy_from_slice(&state_root_bytes);
                return self.get_state(&state_root);
            }
        }
        
        Ok(None)
    }

    fn set_finalized_state(&self, state_root: &Root) -> Result<(), DatabaseError> {
        let key = Self::finalized_key();
        self.database.put(&key, state_root)
    }
}

/// State cache for hot states
#[derive(Debug, Clone)]
pub struct StateCache {
    cache: HashMap<Root, BeaconState>,
    max_size: usize,
}

impl StateCache {
    /// Create new state cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Get state from cache
    pub fn get(&self, state_root: &Root) -> Option<&BeaconState> {
        self.cache.get(state_root)
    }

    /// Put state in cache
    pub fn put(&mut self, state_root: Root, state: BeaconState) {
        // Simple eviction: remove oldest when at capacity
        if self.cache.len() >= self.max_size {
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }
        
        self.cache.insert(state_root, state);
    }

    /// Remove state from cache
    pub fn remove(&mut self, state_root: &Root) {
        self.cache.remove(state_root);
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::DatabaseConfig;

    #[test]
    fn test_state_store_creation() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config).unwrap();
        let store = StateStore::new(db);
        
        // Test basic operations (with placeholder state)
        let state_root = [42u8; 32];
        let exists = store.has_state(&state_root).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_state_cache() {
        let mut cache = StateCache::new(2);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        
        let state_root1 = [1u8; 32];
        let state_root2 = [2u8; 32];
        let state1 = BeaconState::default();
        let state2 = BeaconState::default();
        
        cache.put(state_root1, state1.clone());
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&state_root1).is_some());
        
        cache.put(state_root2, state2.clone());
        assert_eq!(cache.len(), 2);
        
        // Adding third item should evict first
        let state_root3 = [3u8; 32];
        let state3 = BeaconState::default();
        cache.put(state_root3, state3.clone());
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_key_generation() {
        let state_root = [42u8; 32];
        let key = StateStore::state_key(&state_root);
        assert!(key.starts_with(b"state:"));
        assert_eq!(key.len(), 6 + 32);
        
        let slot_key = StateStore::slot_index_key(123);
        assert!(slot_key.starts_with(b"slot_index:"));
        assert_eq!(slot_key.len(), 11 + 8);
    }
}
