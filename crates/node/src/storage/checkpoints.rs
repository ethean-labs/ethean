//! Checkpoint storage — keyed by Lean slot (not Beacon epoch).

use crate::storage::database::{Database, DatabaseError};
use ethean_types::{Root, State};
use serde::{Deserialize, Serialize};

/// Persisted checkpoint metadata (node storage, not consensus container).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub slot: u64,
    pub state_root: Root,
    pub block_root: Root,
    pub timestamp: u64,
    pub size: u64,
}

pub trait CheckpointStore: Send + Sync {
    fn store_checkpoint(&self, checkpoint: &Checkpoint, state: &State) -> Result<(), DatabaseError>;
    fn get_checkpoint(&self, slot: u64) -> Result<Option<Checkpoint>, DatabaseError>;
    fn get_checkpoint_state(&self, slot: u64) -> Result<Option<State>, DatabaseError>;
    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, DatabaseError>;
    fn delete_checkpoint(&self, slot: u64) -> Result<(), DatabaseError>;
    fn get_latest_checkpoint(&self) -> Result<Option<Checkpoint>, DatabaseError>;
    fn prune_checkpoints(&self, keep_count: usize) -> Result<Vec<u64>, DatabaseError>;
}

#[derive(Clone)]
pub struct CheckpointManager {
    database: Database,
}

impl CheckpointManager {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    fn checkpoint_key(slot: u64) -> Vec<u8> {
        let mut key = b"checkpoint:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    fn checkpoint_state_key(slot: u64) -> Vec<u8> {
        let mut key = b"checkpoint_state:".to_vec();
        key.extend_from_slice(&slot.to_be_bytes());
        key
    }

    fn latest_key() -> Vec<u8> {
        b"latest_checkpoint".to_vec()
    }
}

impl CheckpointStore for CheckpointManager {
    fn store_checkpoint(&self, checkpoint: &Checkpoint, state: &State) -> Result<(), DatabaseError> {
        let meta_key = Self::checkpoint_key(checkpoint.slot);
        self.database.put_typed(&meta_key, checkpoint)?;
        let state_key = Self::checkpoint_state_key(checkpoint.slot);
        let bytes = state
            .ssz_encode()
            .map_err(|e| DatabaseError::SerializationError(e.to_string()))?;
        self.database.put(&state_key, &bytes)?;
        self.database
            .put(&Self::latest_key(), &checkpoint.slot.to_be_bytes())?;
        Ok(())
    }

    fn get_checkpoint(&self, slot: u64) -> Result<Option<Checkpoint>, DatabaseError> {
        self.database.get_typed(&Self::checkpoint_key(slot))
    }

    fn get_checkpoint_state(&self, slot: u64) -> Result<Option<State>, DatabaseError> {
        match self.database.get(&Self::checkpoint_state_key(slot))? {
            None => Ok(None),
            Some(bytes) => {
                if bytes.is_empty() {
                    return Err(DatabaseError::SerializationError(
                        "empty checkpoint state; refuse State::default".into(),
                    ));
                }
                State::ssz_decode(&bytes)
                    .map(Some)
                    .map_err(|e| DatabaseError::SerializationError(e.to_string()))
            }
        }
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, DatabaseError> {
        // Full prefix scan deferred; return latest only when present.
        match self.get_latest_checkpoint()? {
            Some(cp) => Ok(vec![cp]),
            None => Ok(Vec::new()),
        }
    }

    fn delete_checkpoint(&self, slot: u64) -> Result<(), DatabaseError> {
        self.database.delete(&Self::checkpoint_key(slot))?;
        self.database.delete(&Self::checkpoint_state_key(slot))?;
        Ok(())
    }

    fn get_latest_checkpoint(&self) -> Result<Option<Checkpoint>, DatabaseError> {
        if let Some(bytes) = self.database.get(&Self::latest_key())? {
            if bytes.len() == 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes);
                let slot = u64::from_be_bytes(arr);
                return self.get_checkpoint(slot);
            }
        }
        Ok(None)
    }

    fn prune_checkpoints(&self, keep_count: usize) -> Result<Vec<u64>, DatabaseError> {
        let mut checkpoints = self.list_checkpoints()?;
        checkpoints.sort_by_key(|c| c.slot);
        let mut deleted = Vec::new();
        while checkpoints.len() > keep_count {
            if let Some(cp) = checkpoints.first().cloned() {
                self.delete_checkpoint(cp.slot)?;
                deleted.push(cp.slot);
                checkpoints.remove(0);
            } else {
                break;
            }
        }
        Ok(deleted)
    }
}
