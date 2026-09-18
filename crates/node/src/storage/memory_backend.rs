//! In-memory database backend for testing
//!
//! Simple HashMap-based storage for unit tests

use crate::storage::database::{DatabaseBackend, DatabaseError, BatchOperation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory database backend
pub struct InMemoryBackend {
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryBackend {
    /// Create new in-memory backend
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl DatabaseBackend for InMemoryBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        Ok(data.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let mut data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        let mut data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        data.remove(key);
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool, DatabaseError> {
        let data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        Ok(data.contains_key(key))
    }

    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, DatabaseError> {
        let data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        let keys: Vec<Vec<u8>> = data
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    fn batch_write(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError> {
        let mut data = self.data.lock().map_err(|_| DatabaseError::Connection("Lock poisoned".to_string()))?;
        
        for op in operations {
            match op {
                BatchOperation::Put { key, value } => {
                    data.insert(key, value);
                }
                BatchOperation::Delete { key } => {
                    data.remove(&key);
                }
            }
        }
        
        Ok(())
    }

    fn close(&self) -> Result<(), DatabaseError> {
        // Nothing to close for in-memory
        Ok(())
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}
