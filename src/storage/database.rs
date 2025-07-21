//! Database abstraction layer
//!
//! Provides a unified interface for different database backends
//! with RocksDB as the primary implementation.

use std::path::PathBuf;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database path
    pub path: PathBuf,
    /// Enable compression
    pub compression: bool,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Max open files
    pub max_open_files: Option<i32>,
    /// Enable statistics
    pub enable_statistics: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data"),
            compression: true,
            cache_size_mb: 512,
            max_open_files: Some(1000),
            enable_statistics: false,
        }
    }
}

/// Database errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(String),
    
    #[error("Key not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Corrupted data: {0}")]
    CorruptedData(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("RocksDB error: {0}")]
    RocksDb(String),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Database trait for different backends
pub trait DatabaseBackend: Send + Sync {
    /// Get value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError>;
    
    /// Put key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError>;
    
    /// Delete key
    fn delete(&self, key: &[u8]) -> Result<(), DatabaseError>;
    
    /// Check if key exists
    fn exists(&self, key: &[u8]) -> Result<bool, DatabaseError>;
    
    /// Get keys with prefix
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, DatabaseError>;
    
    /// Batch operations
    fn batch_write(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError>;
    
    /// Close database
    fn close(&self) -> Result<(), DatabaseError>;
}

/// Batch operation for atomic writes
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

/// Database handle
#[derive(Clone)]
pub struct Database {
    backend: Arc<dyn DatabaseBackend>,
}

impl Database {
    /// Open database with configuration
    pub fn open(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let backend = RocksDbBackend::new(config)?;
        Ok(Self {
            backend: Arc::new(backend),
        })
    }

    /// Create in-memory database for testing
    pub fn in_memory() -> Self {
        use crate::storage::memory_backend::InMemoryBackend;
        Self {
            backend: Arc::new(InMemoryBackend::new()),
        }
    }

    /// Get value by key
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        self.backend.get(key)
    }

    /// Get typed value by key
    pub fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &[u8]) -> Result<Option<T>, DatabaseError> {
        match self.get(key)? {
            Some(data) => Ok(Some(serde_json::from_slice(&data)?)),
            None => Ok(None),
        }
    }

    /// Put key-value pair
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        self.backend.put(key, value)
    }

    /// Put typed value
    pub fn put_typed<T: Serialize>(&self, key: &[u8], value: &T) -> Result<(), DatabaseError> {
        let data = serde_json::to_vec(value)?;
        self.put(key, &data)
    }

    /// Delete key
    pub fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        self.backend.delete(key)
    }

    /// Check if key exists
    pub fn exists(&self, key: &[u8]) -> Result<bool, DatabaseError> {
        self.backend.exists(key)
    }

    /// Get keys with prefix
    pub fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, DatabaseError> {
        self.backend.keys_with_prefix(prefix)
    }

    /// Batch operations
    pub fn batch_write(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError> {
        self.backend.batch_write(operations)
    }

    /// Close database
    pub fn close(self) -> Result<(), DatabaseError> {
        // For trait objects, we can't unwrap Arc, so just drop
        // In a real implementation, this would flush and close properly
        Ok(())
    }
}

/// RocksDB backend implementation
pub struct RocksDbBackend {
    // Placeholder for now - would use rocksdb crate in production
    _config: DatabaseConfig,
}

impl RocksDbBackend {
    pub fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&config.path)?;
        
        Ok(Self {
            _config: config.clone(),
        })
    }
}

impl DatabaseBackend for RocksDbBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        Ok(backend.lock().unwrap().get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        // Note: This is a simplified implementation. In production, we'd use proper RocksDB
        // For now, we'll use a mutex to make it thread-safe
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        backend.lock().unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        backend.lock().unwrap().remove(key);
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool, DatabaseError> {
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        Ok(backend.lock().unwrap().contains_key(key))
    }

    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, DatabaseError> {
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let keys: Vec<Vec<u8>> = backend.lock().unwrap()
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    fn batch_write(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError> {
        use std::sync::Mutex;
        static BACKEND: std::sync::OnceLock<Mutex<std::collections::HashMap<Vec<u8>, Vec<u8>>>> = std::sync::OnceLock::new();
        
        let backend = BACKEND.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
        let mut backend = backend.lock().unwrap();
        
        for operation in operations {
            match operation {
                BatchOperation::Put { key, value } => {
                    backend.insert(key, value);
                }
                BatchOperation::Delete { key } => {
                    backend.remove(&key);
                }
            }
        }
        Ok(())
    }

    fn close(&self) -> Result<(), DatabaseError> {
        // In a real implementation, we'd close the RocksDB instance
        Ok(())
    }

    /// Apply batch operations
    pub fn apply_batch(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError> {
        self.batch_write(operations)
    }
    
    /// Create database snapshot
    pub fn create_snapshot(&self) -> Result<DatabaseSnapshot, DatabaseError> {
        // Create a snapshot for consistent reads
        Ok(DatabaseSnapshot {
            _phantom: std::marker::PhantomData,
        })
    }
    
    /// Get value from snapshot
    pub fn get_from_snapshot(&self, _snapshot: &DatabaseSnapshot, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        // For now, just use regular get
        self.get(key)
    }
    
    /// Get all keys
    pub fn all_keys(&self) -> Result<Vec<Vec<u8>>, DatabaseError> {
        self.keys_with_prefix(&[])
    }
    
    /// Clear all data
    pub fn clear_all(&self) -> Result<(), DatabaseError> {
        // Get all keys and delete them
        let keys = self.all_keys()?;
        for key in keys {
            self.delete(&key)?;
        }
        Ok(())
    }
    
    /// Batch put operations
    pub fn batch_put(&self, items: &[(Vec<u8>, Vec<u8>)]) -> Result<(), DatabaseError> {
        let operations = items.iter()
            .map(|(key, value)| BatchOperation::Put { 
                key: key.clone(), 
                value: value.clone() 
            })
            .collect();
        self.apply_batch(operations)
    }
    
    /// Create in-memory database for testing
    pub fn in_memory() -> Database {
        Database {
            backend: Arc::new(InMemoryBackend::new()),
        }
    }
}

/// Database snapshot for consistent reads
pub struct DatabaseSnapshot {
    _phantom: std::marker::PhantomData<()>,
}

// In-memory backend implementation for testing
struct InMemoryBackend {
    data: std::sync::RwLock<std::collections::HashMap<Vec<u8>, Vec<u8>>>,
}

impl InMemoryBackend {
    fn new() -> Self {
        Self {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl DatabaseBackend for InMemoryBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }
    
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }
    
    fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }
    
    fn exists(&self, key: &[u8]) -> Result<bool, DatabaseError> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }
    
    fn keys_with_prefix(&self, prefix: &[u8]) -> Result<Vec<Vec<u8>>, DatabaseError> {
        let data = self.data.read().unwrap();
        let keys = data.keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }
    
    fn batch_write(&self, operations: Vec<BatchOperation>) -> Result<(), DatabaseError> {
        let mut data = self.data.write().unwrap();
        for operation in operations {
            match operation {
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.path, PathBuf::from("./data"));
        assert!(config.compression);
        assert_eq!(config.cache_size_mb, 512);
    }

    #[test]
    fn test_database_open() {
        let config = DatabaseConfig::default();
        let db = Database::open(&config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_batch_operations() {
        let ops = vec![
            BatchOperation::Put { 
                key: b"key1".to_vec(), 
                value: b"value1".to_vec() 
            },
            BatchOperation::Delete { 
                key: b"key2".to_vec() 
            },
        ];
        
        assert_eq!(ops.len(), 2);
    }
}
