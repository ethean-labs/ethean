//! Advanced caching layer for database optimization
//! 
//! Implements LRU cache with async support, memory management,
//! and performance monitoring for database operations.

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Entry time-to-live
    pub ttl: Duration,
    /// Enable cache statistics
    pub enable_stats: bool,
    /// Cache write-through vs write-back
    pub write_through: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            max_memory_bytes: 256 * 1024 * 1024, // 256MB
            ttl: Duration::from_secs(3600), // 1 hour
            enable_stats: true,
            write_through: true,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    size_bytes: usize,
}

impl CacheEntry {
    fn new(data: Vec<u8>) -> Self {
        let now = Instant::now();
        let size_bytes = data.len();
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_bytes,
        }
    }
    
    fn access(&mut self) -> &[u8] {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }
    
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub memory_bytes: usize,
    pub evictions: u64,
    pub expired_entries: u64,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f64 {
        if self.hits + self.misses == 0 {
            return 0.0;
        }
        self.hits as f64 / (self.hits + self.misses) as f64
    }
    
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// LRU Cache implementation with async support
pub struct LruCache {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<Vec<u8>, CacheEntry>>>,
    access_order: Arc<RwLock<Vec<Vec<u8>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl LruCache {
    /// Create new LRU cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Get value from cache
    pub async fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut entries = self.entries.write().await;
        let mut access_order = self.access_order.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            // Check if expired
            if entry.is_expired(self.config.ttl) {
                let key_vec = key.to_vec();
                entries.remove(&key_vec);
                access_order.retain(|k| k != &key_vec);
                
                if self.config.enable_stats {
                    let mut stats = self.stats.write().await;
                    stats.expired_entries += 1;
                    stats.entries = entries.len();
                    stats.memory_bytes = entries.values().map(|e| e.size_bytes).sum();
                }
                
                return None;
            }
            
            // Update access order (move to end)
            let key_vec = key.to_vec();
            access_order.retain(|k| k != &key_vec);
            access_order.push(key_vec);
            
            // Update stats
            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.hits += 1;
            }
            
            return Some(entry.access().to_vec());
        }
        
        // Cache miss
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
        }
        
        None
    }
    
    /// Put value into cache
    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) {
        let mut entries = self.entries.write().await;
        let mut access_order = self.access_order.write().await;
        
        // Create new entry
        let entry = CacheEntry::new(value);
        let entry_size = entry.size_bytes;
        
        // Check if we need to evict entries
        self.evict_if_needed(&mut entries, &mut access_order, entry_size).await;
        
        // Remove existing entry from access order if it exists
        access_order.retain(|k| k != &key);
        
        // Add new entry
        entries.insert(key.clone(), entry);
        access_order.push(key);
        
        // Update stats
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.entries = entries.len();
            stats.memory_bytes = entries.values().map(|e| e.size_bytes).sum();
        }
    }
    
    /// Remove value from cache
    pub async fn remove(&self, key: &[u8]) -> bool {
        let mut entries = self.entries.write().await;
        let mut access_order = self.access_order.write().await;
        
        let key_vec = key.to_vec();
        let removed = entries.remove(&key_vec).is_some();
        
        if removed {
            access_order.retain(|k| k != &key_vec);
            
            if self.config.enable_stats {
                let mut stats = self.stats.write().await;
                stats.entries = entries.len();
                stats.memory_bytes = entries.values().map(|e| e.size_bytes).sum();
            }
        }
        
        removed
    }
    
    /// Clear all cached entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let mut access_order = self.access_order.write().await;
        
        entries.clear();
        access_order.clear();
        
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.entries = 0;
            stats.memory_bytes = 0;
        }
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
    
    /// Evict entries if needed to make room
    async fn evict_if_needed(
        &self,
        entries: &mut HashMap<Vec<u8>, CacheEntry>,
        access_order: &mut Vec<Vec<u8>>,
        new_entry_size: usize,
    ) {
        let current_memory: usize = entries.values().map(|e| e.size_bytes).sum();
        let mut evictions = 0;
        
        // Check memory limit
        while (current_memory + new_entry_size) > self.config.max_memory_bytes && !access_order.is_empty() {
            if let Some(oldest_key) = access_order.first().cloned() {
                entries.remove(&oldest_key);
                access_order.remove(0);
                evictions += 1;
            } else {
                break;
            }
        }
        
        // Check entry count limit
        while entries.len() >= self.config.max_entries && !access_order.is_empty() {
            if let Some(oldest_key) = access_order.first().cloned() {
                entries.remove(&oldest_key);
                access_order.remove(0);
                evictions += 1;
            } else {
                break;
            }
        }
        
        // Update stats
        if self.config.enable_stats && evictions > 0 {
            let mut stats = self.stats.write().await;
            stats.evictions += evictions;
        }
    }
    
    /// Cleanup expired entries (background task)
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let mut access_order = self.access_order.write().await;
        let mut expired_count = 0;
        
        let expired_keys: Vec<Vec<u8>> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.config.ttl))
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            entries.remove(&key);
            access_order.retain(|k| k != &key);
            expired_count += 1;
        }
        
        if self.config.enable_stats && expired_count > 0 {
            let mut stats = self.stats.write().await;
            stats.expired_entries += expired_count;
            stats.entries = entries.len();
            stats.memory_bytes = entries.values().map(|e| e.size_bytes).sum();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            max_entries: 2,
            max_memory_bytes: 1024,
            ttl: Duration::from_secs(1),
            enable_stats: true,
            write_through: true,
        };
        
        let cache = LruCache::new(config);
        
        // Test put and get
        cache.put(b"key1".to_vec(), b"value1".to_vec()).await;
        assert_eq!(cache.get(b"key1").await, Some(b"value1".to_vec()));
        
        // Test cache miss
        assert_eq!(cache.get(b"key2").await, None);
        
        // Test LRU eviction
        cache.put(b"key2".to_vec(), b"value2".to_vec()).await;
        cache.put(b"key3".to_vec(), b"value3".to_vec()).await; // Should evict key1
        
        assert_eq!(cache.get(b"key1").await, None);
        assert_eq!(cache.get(b"key2").await, Some(b"value2".to_vec()));
        assert_eq!(cache.get(b"key3").await, Some(b"value3".to_vec()));
    }
    
    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            max_entries: 10,
            max_memory_bytes: 1024,
            ttl: Duration::from_millis(100),
            enable_stats: true,
            write_through: true,
        };
        
        let cache = LruCache::new(config);
        
        cache.put(b"key1".to_vec(), b"value1".to_vec()).await;
        assert_eq!(cache.get(b"key1").await, Some(b"value1".to_vec()));
        
        // Wait for expiration
        sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get(b"key1").await, None);
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = LruCache::new(config);
        
        cache.put(b"key1".to_vec(), b"value1".to_vec()).await;
        cache.get(b"key1").await; // Hit
        cache.get(b"key2").await; // Miss
        
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entries, 1);
        assert!(stats.hit_ratio() > 0.0);
    }
}
