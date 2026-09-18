//! Database indexing optimization module
//!
//! Provides advanced indexing strategies for efficient data retrieval,
//! including composite indexes, range queries, and performance optimization.

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use crate::types::{Slot, Epoch, ValidatorIndex, Root};
use crate::storage::database::{Database, DatabaseError};

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Enable automatic index maintenance
    pub auto_maintain: bool,
    /// Index cache size
    pub cache_size: usize,
    /// Rebuild threshold (number of operations before rebuild)
    pub rebuild_threshold: usize,
    /// Enable index statistics
    pub enable_stats: bool,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            auto_maintain: true,
            cache_size: 10000,
            rebuild_threshold: 10000,
            enable_stats: true,
        }
    }
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub total_keys: usize,
    pub index_hits: u64,
    pub index_misses: u64,
    pub rebuilds: u64,
    pub last_rebuild: Option<std::time::Instant>,
}

/// Index type definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// Slot-based index for blocks
    SlotIndex,
    /// Epoch-based index for checkpoint data
    EpochIndex,
    /// Validator index for validator-specific data
    ValidatorIndex,
    /// Root-based index for state and block lookups
    RootIndex,
    /// Composite index combining multiple fields
    CompositeIndex(Vec<String>),
}

/// Index key structure
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexKey {
    Slot(Slot),
    Epoch(Epoch),
    Validator(ValidatorIndex),
    Root(Root),
    Composite(Vec<Vec<u8>>),
}

impl IndexKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IndexKey::Slot(slot) => slot.to_le_bytes().to_vec(),
            IndexKey::Epoch(epoch) => epoch.to_le_bytes().to_vec(),
            IndexKey::Validator(validator) => validator.to_le_bytes().to_vec(),
            IndexKey::Root(root) => root.0.to_vec(),
            IndexKey::Composite(parts) => {
                let mut result = Vec::new();
                for part in parts {
                    result.extend_from_slice(&(part.len() as u32).to_le_bytes());
                    result.extend_from_slice(part);
                }
                result
            }
        }
    }
    
    pub fn from_bytes(index_type: &IndexType, data: &[u8]) -> Result<Self, DatabaseError> {
        match index_type {
            IndexType::SlotIndex => {
                if data.len() != 8 {
                    return Err(DatabaseError::InvalidData("Invalid slot data length".to_string()));
                }
                let slot = u64::from_le_bytes(data.try_into().unwrap());
                Ok(IndexKey::Slot(slot))
            }
            IndexType::EpochIndex => {
                if data.len() != 8 {
                    return Err(DatabaseError::InvalidData("Invalid epoch data length".to_string()));
                }
                let epoch = u64::from_le_bytes(data.try_into().unwrap());
                Ok(IndexKey::Epoch(epoch))
            }
            IndexType::ValidatorIndex => {
                if data.len() != 8 {
                    return Err(DatabaseError::InvalidData("Invalid validator data length".to_string()));
                }
                let validator = u64::from_le_bytes(data.try_into().unwrap());
                Ok(IndexKey::Validator(validator))
            }
            IndexType::RootIndex => {
                if data.len() != 32 {
                    return Err(DatabaseError::InvalidData("Invalid root data length".to_string()));
                }
                let mut root_bytes = [0u8; 32];
                root_bytes.copy_from_slice(data);
                Ok(IndexKey::Root(Root(root_bytes)))
            }
            IndexType::CompositeIndex(_) => {
                let mut parts = Vec::new();
                let mut offset = 0;
                
                while offset < data.len() {
                    if offset + 4 > data.len() {
                        return Err(DatabaseError::InvalidData("Invalid composite index data".to_string()));
                    }
                    
                    let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
                    offset += 4;
                    
                    if offset + len > data.len() {
                        return Err(DatabaseError::InvalidData("Invalid composite index data".to_string()));
                    }
                    
                    parts.push(data[offset..offset + len].to_vec());
                    offset += len;
                }
                
                Ok(IndexKey::Composite(parts))
            }
        }
    }
}

/// Range query parameters
#[derive(Debug, Clone)]
pub struct RangeQuery {
    pub start: Option<IndexKey>,
    pub end: Option<IndexKey>,
    pub limit: Option<usize>,
    pub ascending: bool,
}

impl Default for RangeQuery {
    fn default() -> Self {
        Self {
            start: None,
            end: None,
            limit: None,
            ascending: true,
        }
    }
}

/// Index implementation
pub struct DatabaseIndex {
    index_type: IndexType,
    config: IndexConfig,
    // BTreeMap for ordered access and range queries
    index: Arc<RwLock<BTreeMap<IndexKey, Vec<Vec<u8>>>>>,
    stats: Arc<RwLock<IndexStats>>,
    database: Arc<Database>,
    operations_count: Arc<RwLock<usize>>,
}

impl DatabaseIndex {
    /// Create new index
    pub fn new(
        index_type: IndexType,
        config: IndexConfig,
        database: Arc<Database>,
    ) -> Self {
        Self {
            index_type,
            config,
            index: Arc::new(RwLock::new(BTreeMap::new())),
            stats: Arc::new(RwLock::new(IndexStats::default())),
            database,
            operations_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Add entry to index
    pub async fn add_entry(&self, key: IndexKey, value: Vec<u8>) -> Result<(), DatabaseError> {
        let mut index = self.index.write().await;
        
        // Add to index
        index.entry(key).or_insert_with(Vec::new).push(value);
        
        // Update operation count
        let mut ops_count = self.operations_count.write().await;
        *ops_count += 1;
        
        // Check if rebuild is needed
        if self.config.auto_maintain && *ops_count >= self.config.rebuild_threshold {
            drop(index);
            drop(ops_count);
            self.rebuild().await?;
        }
        
        Ok(())
    }
    
    /// Remove entry from index
    pub async fn remove_entry(&self, key: &IndexKey, value: &[u8]) -> Result<bool, DatabaseError> {
        let mut index = self.index.write().await;
        
        if let Some(values) = index.get_mut(key) {
            let initial_len = values.len();
            values.retain(|v| v != value);
            
            // Remove key if no values left
            if values.is_empty() {
                index.remove(key);
            }
            
            // Update operation count
            let mut ops_count = self.operations_count.write().await;
            *ops_count += 1;
            
            return Ok(values.len() != initial_len);
        }
        
        Ok(false)
    }
    
    /// Get entries by key
    pub async fn get_entries(&self, key: &IndexKey) -> Result<Vec<Vec<u8>>, DatabaseError> {
        let index = self.index.read().await;
        
        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            if index.contains_key(key) {
                stats.index_hits += 1;
            } else {
                stats.index_misses += 1;
            }
        }
        
        Ok(index.get(key).cloned().unwrap_or_default())
    }
    
    /// Range query
    pub async fn range_query(&self, query: RangeQuery) -> Result<Vec<(IndexKey, Vec<Vec<u8>>)>, DatabaseError> {
        let index = self.index.read().await;
        let mut results = Vec::new();
        
        let iter: Box<dyn Iterator<Item = (&IndexKey, &Vec<Vec<u8>>)>> = if query.ascending {
            Box::new(index.iter())
        } else {
            Box::new(index.iter().rev())
        };
        
        let mut count = 0;
        for (key, values) in iter {
            // Check start bound
            if let Some(ref start) = query.start {
                if query.ascending && key < start {
                    continue;
                }
                if !query.ascending && key > start {
                    continue;
                }
            }
            
            // Check end bound
            if let Some(ref end) = query.end {
                if query.ascending && key > end {
                    break;
                }
                if !query.ascending && key < end {
                    break;
                }
            }
            
            results.push((key.clone(), values.clone()));
            count += 1;
            
            // Check limit
            if let Some(limit) = query.limit {
                if count >= limit {
                    break;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get keys in range
    pub async fn keys_in_range(&self, start: Option<IndexKey>, end: Option<IndexKey>) -> Result<Vec<IndexKey>, DatabaseError> {
        let query = RangeQuery {
            start,
            end,
            limit: None,
            ascending: true,
        };
        
        let results = self.range_query(query).await?;
        Ok(results.into_iter().map(|(key, _)| key).collect())
    }
    
    /// Rebuild index from database
    pub async fn rebuild(&self) -> Result<(), DatabaseError> {
        let mut index = self.index.write().await;
        let mut operations_count = self.operations_count.write().await;
        
        // Clear current index
        index.clear();
        *operations_count = 0;
        
        // Get all keys from database for this index type
        let prefix = self.get_index_prefix();
        let keys = self.database.keys_with_prefix(&prefix)?;
        
        // Rebuild index
        for key_bytes in keys {
            if let Ok(value) = self.database.get(&key_bytes)? {
                if let Some(value_bytes) = value {
                    // Extract index key from database key
                    if let Ok(index_key) = IndexKey::from_bytes(&self.index_type, &key_bytes[prefix.len()..]) {
                        index.entry(index_key).or_insert_with(Vec::new).push(value_bytes);
                    }
                }
            }
        }
        
        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.rebuilds += 1;
            stats.last_rebuild = Some(std::time::Instant::now());
            stats.total_keys = index.len();
        }
        
        Ok(())
    }
    
    /// Get index statistics
    pub async fn stats(&self) -> IndexStats {
        let mut stats = self.stats.read().await.clone();
        let index = self.index.read().await;
        stats.total_keys = index.len();
        stats
    }
    
    /// Clear index
    pub async fn clear(&self) -> Result<(), DatabaseError> {
        let mut index = self.index.write().await;
        let mut operations_count = self.operations_count.write().await;
        
        index.clear();
        *operations_count = 0;
        
        Ok(())
    }
    
    /// Get index prefix for database keys
    fn get_index_prefix(&self) -> Vec<u8> {
        match &self.index_type {
            IndexType::SlotIndex => b"slot_idx:".to_vec(),
            IndexType::EpochIndex => b"epoch_idx:".to_vec(),
            IndexType::ValidatorIndex => b"validator_idx:".to_vec(),
            IndexType::RootIndex => b"root_idx:".to_vec(),
            IndexType::CompositeIndex(fields) => {
                let mut prefix = b"composite_idx:".to_vec();
                for field in fields {
                    prefix.extend_from_slice(field.as_bytes());
                    prefix.push(b':');
                }
                prefix
            }
        }
    }
}

/// Index manager for multiple indexes
pub struct IndexManager {
    indexes: Arc<RwLock<HashMap<String, DatabaseIndex>>>,
    database: Arc<Database>,
}

impl IndexManager {
    /// Create new index manager
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            database,
        }
    }
    
    /// Create index
    pub async fn create_index(
        &self,
        name: String,
        index_type: IndexType,
        config: IndexConfig,
    ) -> Result<(), DatabaseError> {
        let mut indexes = self.indexes.write().await;
        let index = DatabaseIndex::new(index_type, config, self.database.clone());
        indexes.insert(name, index);
        Ok(())
    }
    
    /// Get index by name
    pub async fn get_index(&self, name: &str) -> Option<DatabaseIndex> {
        let indexes = self.indexes.read().await;
        indexes.get(name).cloned()
    }
    
    /// Drop index
    pub async fn drop_index(&self, name: &str) -> Result<bool, DatabaseError> {
        let mut indexes = self.indexes.write().await;
        Ok(indexes.remove(name).is_some())
    }
    
    /// List all indexes
    pub async fn list_indexes(&self) -> Vec<String> {
        let indexes = self.indexes.read().await;
        indexes.keys().cloned().collect()
    }
    
    /// Rebuild all indexes
    pub async fn rebuild_all(&self) -> Result<(), DatabaseError> {
        let indexes = self.indexes.read().await;
        
        for index in indexes.values() {
            index.rebuild().await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    
    #[tokio::test]
    async fn test_slot_index() {
        let db = Arc::new(Database::in_memory());
        let config = IndexConfig::default();
        let index = DatabaseIndex::new(IndexType::SlotIndex, config, db);
        
        // Add entries
        index.add_entry(IndexKey::Slot(100), b"block_100".to_vec()).await.unwrap();
        index.add_entry(IndexKey::Slot(101), b"block_101".to_vec()).await.unwrap();
        index.add_entry(IndexKey::Slot(100), b"extra_data_100".to_vec()).await.unwrap();
        
        // Test get entries
        let entries = index.get_entries(&IndexKey::Slot(100)).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&b"block_100".to_vec()));
        assert!(entries.contains(&b"extra_data_100".to_vec()));
        
        // Test range query
        let query = RangeQuery {
            start: Some(IndexKey::Slot(100)),
            end: Some(IndexKey::Slot(101)),
            limit: None,
            ascending: true,
        };
        
        let results = index.range_query(query).await.unwrap();
        assert_eq!(results.len(), 2);
    }
    
    #[tokio::test]
    async fn test_composite_index() {
        let db = Arc::new(Database::in_memory());
        let config = IndexConfig::default();
        let index = DatabaseIndex::new(
            IndexType::CompositeIndex(vec!["epoch".to_string(), "validator".to_string()]),
            config,
            db,
        );
        
        // Add composite entries
        let key1 = IndexKey::Composite(vec![
            100u64.to_le_bytes().to_vec(), // epoch
            42u64.to_le_bytes().to_vec(),  // validator
        ]);
        
        index.add_entry(key1.clone(), b"attestation_data".to_vec()).await.unwrap();
        
        let entries = index.get_entries(&key1).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], b"attestation_data".to_vec());
    }
}
