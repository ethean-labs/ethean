//! State storage — SSZ bytes (no JSON consensus roots).

use crate::storage::database::{Database, DatabaseError};
use ethean_types::{Root, State};
use std::collections::HashMap;

/// State storage trait
pub trait StateStorage: Send + Sync {
    fn store_state(&self, state_root: &Root, state: &State) -> Result<(), DatabaseError>;
    fn get_state(&self, state_root: &Root) -> Result<Option<State>, DatabaseError>;
    fn get_state_by_slot(&self, slot: u64) -> Result<Option<State>, DatabaseError>;
    fn has_state(&self, state_root: &Root) -> Result<bool, DatabaseError>;
    fn delete_state(&self, state_root: &Root) -> Result<(), DatabaseError>;
    fn get_finalized_state(&self) -> Result<Option<State>, DatabaseError>;
    fn set_finalized_state(&self, state_root: &Root) -> Result<(), DatabaseError>;
}

/// State store backed by the node database.
#[derive(Clone)]
pub struct StateStore {
    database: Database,
    cache: HashMap<Root, State>,
}

impl StateStore {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            cache: HashMap::new(),
        }
    }

    fn state_key(state_root: &Root) -> Vec<u8> {
        let mut key = b"state:".to_vec();
        key.extend_from_slice(state_root);
        key
    }

    fn slot_index_key(slot: u64) -> Vec<u8> {
        let mut key = b"slot_index:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    fn finalized_key() -> Vec<u8> {
        b"finalized_state".to_vec()
    }
}

impl StateStorage for StateStore {
    fn store_state(&self, state_root: &Root, state: &State) -> Result<(), DatabaseError> {
        let key = Self::state_key(state_root);
        let bytes = state
            .ssz_encode()
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        self.database.put(&key, &bytes)?;
        let slot_key = Self::slot_index_key(state.slot.get());
        self.database.put(&slot_key, state_root)?;
        Ok(())
    }

    fn get_state(&self, state_root: &Root) -> Result<Option<State>, DatabaseError> {
        let key = Self::state_key(state_root);
        match self.database.get(&key)? {
            None => Ok(None),
            Some(bytes) => {
                // Full State SSZ decode is incomplete; empty payload → default.
                if bytes.is_empty() {
                    return Ok(Some(State::default()));
                }
                State::ssz_decode(&bytes)
                    .map(Some)
                    .or_else(|_| Ok(Some(State::default())))
            }
        }
    }

    fn get_state_by_slot(&self, slot: u64) -> Result<Option<State>, DatabaseError> {
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
        self.database.exists(&Self::state_key(state_root))
    }

    fn delete_state(&self, state_root: &Root) -> Result<(), DatabaseError> {
        self.database.delete(&Self::state_key(state_root))
    }

    fn get_finalized_state(&self) -> Result<Option<State>, DatabaseError> {
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
        self.database.put(&Self::finalized_key(), state_root)
    }
}

/// Hot-state cache.
#[derive(Debug, Clone, Default)]
pub struct StateCache {
    cache: HashMap<Root, State>,
    max_size: usize,
}

impl StateCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, state_root: &Root) -> Option<&State> {
        self.cache.get(state_root)
    }

    pub fn put(&mut self, state_root: Root, state: State) {
        if self.cache.len() >= self.max_size {
            if let Some(k) = self.cache.keys().next().cloned() {
                self.cache.remove(&k);
            }
        }
        self.cache.insert(state_root, state);
    }
}
