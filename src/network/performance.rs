//! Network Performance Optimization Module
//!
//! Implements advanced performance optimizations including message batching,
//! compression, caching, and intelligent routing for maximum throughput.

use crate::network::{NetworkError, ConnectionInfo};
use libp2p::{Multiaddr, PeerId};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Performance optimization manager
pub struct PerformanceOptimizer {
    /// Message batching system
    message_batcher: MessageBatcher,
    /// Compression manager
    compression_manager: CompressionManager,
    /// Caching system
    cache_manager: CacheManager,
    /// Intelligent routing
    routing_optimizer: RoutingOptimizer,
    /// Performance configuration
    config: PerformanceConfig,
    /// Performance metrics
    metrics: PerformanceMetrics,
}

/// Performance optimization configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable message batching
    pub enable_batching: bool,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Batch timeout
    pub batch_timeout: Duration,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression threshold (bytes)
    pub compression_threshold: usize,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache size limit
    pub cache_size_limit: usize,
    /// Cache TTL
    pub cache_ttl: Duration,
    /// Enable intelligent routing
    pub enable_intelligent_routing: bool,
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
}

/// Performance metrics
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    /// Messages batched
    pub messages_batched: u64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average latency
    pub average_latency: Duration,
    /// Throughput (bytes/sec)
    pub throughput: u64,
    /// CPU usage
    pub cpu_usage: f64,
    /// Memory usage
    pub memory_usage: u64,
    /// Network utilization
    pub network_utilization: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_batching: true,
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(10),
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            enable_caching: true,
            cache_size_limit: 10000,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_intelligent_routing: true,
            monitoring_interval: Duration::from_secs(5),
        }
    }
}

impl PerformanceOptimizer {
    /// Create new performance optimizer
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            message_batcher: MessageBatcher::new(config.max_batch_size, config.batch_timeout),
            compression_manager: CompressionManager::new(config.compression_threshold),
            cache_manager: CacheManager::new(config.cache_size_limit, config.cache_ttl),
            routing_optimizer: RoutingOptimizer::new(),
            config,
            metrics: PerformanceMetrics::default(),
        }
    }

    /// Optimize outgoing message
    pub async fn optimize_outgoing_message(
        &mut self,
        message: NetworkMessage,
        target_peer: PeerId,
    ) -> Result<OptimizedMessage, PerformanceError> {
        let mut optimized = OptimizedMessage {
            original_size: message.data.len(),
            compressed_data: None,
            batched: false,
            routing_info: None,
            peer_id: target_peer,
            message_type: message.message_type.clone(),
            timestamp: Instant::now(),
        };

        // Apply compression if enabled and beneficial
        if self.config.enable_compression && message.data.len() > self.config.compression_threshold {
            match self.compression_manager.compress(&message.data).await {
                Ok(compressed) => {
                    if compressed.len() < message.data.len() {
                        optimized.compressed_data = Some(compressed);
                        let ratio = compressed.len() as f64 / message.data.len() as f64;
                        self.update_compression_ratio(ratio);
                    }
                }
                Err(e) => warn!("Compression failed: {}", e),
            }
        }

        // Apply batching if enabled
        if self.config.enable_batching {
            if let Some(batch) = self.message_batcher.add_message(message, target_peer).await {
                optimized.batched = true;
                self.metrics.messages_batched += batch.messages.len() as u64;
                return Ok(optimized);
            }
        }

        // Apply intelligent routing
        if self.config.enable_intelligent_routing {
            optimized.routing_info = Some(
                self.routing_optimizer.get_optimal_route(target_peer).await
            );
        }

        Ok(optimized)
    }

    /// Optimize incoming message processing
    pub async fn process_incoming_message(
        &mut self,
        data: Vec<u8>,
        from_peer: PeerId,
    ) -> Result<Vec<NetworkMessage>, PerformanceError> {
        let start_time = Instant::now();
        
        // Check cache first
        if self.config.enable_caching {
            if let Some(cached) = self.cache_manager.get(&data).await {
                self.update_cache_metrics(true);
                return Ok(vec![cached]);
            }
            self.update_cache_metrics(false);
        }

        // Decompress if needed
        let processed_data = if self.is_compressed(&data) {
            self.compression_manager.decompress(&data).await?
        } else {
            data
        };

        // Check if it's a batch
        let messages = if self.is_batched(&processed_data) {
            self.message_batcher.extract_batch(&processed_data).await?
        } else {
            vec![self.parse_single_message(processed_data)?]
        };

        // Cache the result
        if self.config.enable_caching && messages.len() == 1 {
            self.cache_manager.put(data, messages[0].clone()).await;
        }

        // Update latency metrics
        let latency = start_time.elapsed();
        self.update_latency_metrics(latency);

        Ok(messages)
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Update performance metrics periodically
    pub async fn update_metrics(&mut self) {
        // Update cache hit rate
        self.metrics.cache_hit_rate = self.cache_manager.get_hit_rate();
        
        // Update compression ratio
        self.metrics.compression_ratio = self.compression_manager.get_average_ratio();
        
        // Update throughput and network utilization
        self.calculate_throughput_metrics().await;
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&mut self) {
        let mut interval = interval(self.config.monitoring_interval);
        
        loop {
            interval.tick().await;
            self.update_metrics().await;
            
            // Log performance metrics
            debug!("Performance metrics: {:?}", self.metrics);
            
            // Trigger optimization adjustments if needed
            self.adjust_optimizations().await;
        }
    }

    // Helper methods
    fn is_compressed(&self, data: &[u8]) -> bool {
        // Simple magic number check for compressed data
        data.starts_with(&[0x78, 0x9C]) || data.starts_with(&[0x1F, 0x8B])
    }

    fn is_batched(&self, data: &[u8]) -> bool {
        // Check for batch magic number
        data.starts_with(b"BATCH")
    }

    fn parse_single_message(&self, data: Vec<u8>) -> Result<NetworkMessage, PerformanceError> {
        // Simplified message parsing
        Ok(NetworkMessage {
            message_type: MessageType::Data,
            data,
            timestamp: Instant::now(),
        })
    }

    fn update_compression_ratio(&mut self, ratio: f64) {
        self.metrics.compression_ratio = 
            (self.metrics.compression_ratio * 0.9) + (ratio * 0.1);
    }

    fn update_cache_metrics(&mut self, hit: bool) {
        // Exponential moving average for cache hit rate
        let new_rate = if hit { 1.0 } else { 0.0 };
        self.metrics.cache_hit_rate = 
            (self.metrics.cache_hit_rate * 0.95) + (new_rate * 0.05);
    }

    fn update_latency_metrics(&mut self, latency: Duration) {
        // Exponential moving average for latency
        let current_ms = self.metrics.average_latency.as_millis() as f64;
        let new_ms = latency.as_millis() as f64;
        let avg_ms = (current_ms * 0.9) + (new_ms * 0.1);
        self.metrics.average_latency = Duration::from_millis(avg_ms as u64);
    }

    async fn calculate_throughput_metrics(&mut self) {
        // Simplified throughput calculation
        // In production, this would measure actual bytes transferred
        self.metrics.throughput = 1024 * 1024; // 1MB/s placeholder
        self.metrics.network_utilization = 0.5; // 50% placeholder
    }

    async fn adjust_optimizations(&mut self) {
        // Dynamically adjust optimization parameters based on metrics
        
        // Adjust batch size based on latency
        if self.metrics.average_latency > Duration::from_millis(50) {
            self.message_batcher.increase_batch_size();
        } else if self.metrics.average_latency < Duration::from_millis(10) {
            self.message_batcher.decrease_batch_size();
        }

        // Adjust compression threshold based on CPU usage
        if self.metrics.cpu_usage > 80.0 {
            self.compression_manager.increase_threshold();
        } else if self.metrics.cpu_usage < 50.0 {
            self.compression_manager.decrease_threshold();
        }
    }
}

/// Message batching system
pub struct MessageBatcher {
    max_batch_size: usize,
    batch_timeout: Duration,
    pending_batches: HashMap<PeerId, PendingBatch>,
}

#[derive(Debug)]
struct PendingBatch {
    messages: Vec<NetworkMessage>,
    created_at: Instant,
}

impl MessageBatcher {
    pub fn new(max_batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            max_batch_size,
            batch_timeout,
            pending_batches: HashMap::new(),
        }
    }

    pub async fn add_message(
        &mut self,
        message: NetworkMessage,
        target_peer: PeerId,
    ) -> Option<MessageBatch> {
        let batch = self.pending_batches.entry(target_peer).or_insert_with(|| PendingBatch {
            messages: Vec::new(),
            created_at: Instant::now(),
        });

        batch.messages.push(message);

        // Check if batch is ready
        if batch.messages.len() >= self.max_batch_size ||
           batch.created_at.elapsed() >= self.batch_timeout {
            let messages = std::mem::take(&mut batch.messages);
            self.pending_batches.remove(&target_peer);
            Some(MessageBatch {
                messages,
                target_peer,
                created_at: Instant::now(),
            })
        } else {
            None
        }
    }

    pub async fn extract_batch(&self, data: &[u8]) -> Result<Vec<NetworkMessage>, PerformanceError> {
        // Simplified batch extraction
        // In production, this would properly deserialize batched messages
        Ok(vec![NetworkMessage {
            message_type: MessageType::Data,
            data: data.to_vec(),
            timestamp: Instant::now(),
        }])
    }

    pub fn increase_batch_size(&mut self) {
        self.max_batch_size = (self.max_batch_size + 10).min(1000);
    }

    pub fn decrease_batch_size(&mut self) {
        self.max_batch_size = (self.max_batch_size.saturating_sub(10)).max(10);
    }
}

/// Compression management
pub struct CompressionManager {
    threshold: usize,
    compression_stats: CompressionStats,
}

#[derive(Debug, Default)]
struct CompressionStats {
    total_compressions: u64,
    total_ratio: f64,
}

impl CompressionManager {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            compression_stats: CompressionStats::default(),
        }
    }

    pub async fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>, PerformanceError> {
        if data.len() < self.threshold {
            return Err(PerformanceError::CompressionNotBeneficial);
        }

        // Simplified compression using deflate
        use flate2::Compression;
        use flate2::write::DeflateEncoder;
        use std::io::Write;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| PerformanceError::CompressionFailed(e.to_string()))?;
        let compressed = encoder.finish()
            .map_err(|e| PerformanceError::CompressionFailed(e.to_string()))?;

        // Update statistics
        let ratio = compressed.len() as f64 / data.len() as f64;
        self.compression_stats.total_compressions += 1;
        self.compression_stats.total_ratio += ratio;

        Ok(compressed)
    }

    pub async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, PerformanceError> {
        use flate2::read::DeflateDecoder;
        use std::io::Read;

        let mut decoder = DeflateDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| PerformanceError::DecompressionFailed(e.to_string()))?;

        Ok(decompressed)
    }

    pub fn get_average_ratio(&self) -> f64 {
        if self.compression_stats.total_compressions == 0 {
            1.0
        } else {
            self.compression_stats.total_ratio / self.compression_stats.total_compressions as f64
        }
    }

    pub fn increase_threshold(&mut self) {
        self.threshold = (self.threshold + 512).min(10240); // Max 10KB
    }

    pub fn decrease_threshold(&mut self) {
        self.threshold = (self.threshold.saturating_sub(512)).max(256); // Min 256B
    }
}

/// Caching system
pub struct CacheManager {
    cache: HashMap<Vec<u8>, CacheEntry>,
    size_limit: usize,
    ttl: Duration,
    hit_count: u64,
    miss_count: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    message: NetworkMessage,
    created_at: Instant,
    access_count: u64,
}

impl CacheManager {
    pub fn new(size_limit: usize, ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            size_limit,
            ttl,
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub async fn get(&mut self, key: &[u8]) -> Option<NetworkMessage> {
        // Clean expired entries first
        self.cleanup_expired().await;

        if let Some(entry) = self.cache.get_mut(key) {
            entry.access_count += 1;
            self.hit_count += 1;
            Some(entry.message.clone())
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub async fn put(&mut self, key: Vec<u8>, message: NetworkMessage) {
        // Ensure cache size limit
        if self.cache.len() >= self.size_limit {
            self.evict_lru().await;
        }

        let entry = CacheEntry {
            message,
            created_at: Instant::now(),
            access_count: 1,
        };

        self.cache.insert(key, entry);
    }

    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }

    async fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| {
            now.duration_since(entry.created_at) < self.ttl
        });
    }

    async fn evict_lru(&mut self) {
        // Find least recently used entry (lowest access count)
        if let Some((key_to_remove, _)) = self.cache.iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.cache.remove(&key_to_remove);
        }
    }
}

/// Intelligent routing optimization
pub struct RoutingOptimizer {
    peer_performance: HashMap<PeerId, PeerPerformance>,
    routing_table: HashMap<PeerId, Vec<RoutingPath>>,
}

#[derive(Debug, Clone)]
struct PeerPerformance {
    latency: Duration,
    throughput: u64,
    reliability: f64,
    last_updated: Instant,
}

#[derive(Debug, Clone)]
struct RoutingPath {
    hops: Vec<PeerId>,
    estimated_latency: Duration,
    reliability_score: f64,
}

impl RoutingOptimizer {
    pub fn new() -> Self {
        Self {
            peer_performance: HashMap::new(),
            routing_table: HashMap::new(),
        }
    }

    pub async fn get_optimal_route(&self, target_peer: PeerId) -> RoutingInfo {
        // Simplified routing optimization
        RoutingInfo {
            target_peer,
            preferred_path: vec![target_peer], // Direct connection
            backup_paths: Vec::new(),
            estimated_latency: Duration::from_millis(50),
            reliability_score: 0.95,
        }
    }

    pub async fn update_peer_performance(
        &mut self,
        peer_id: PeerId,
        latency: Duration,
        throughput: u64,
        reliability: f64,
    ) {
        let performance = PeerPerformance {
            latency,
            throughput,
            reliability,
            last_updated: Instant::now(),
        };
        self.peer_performance.insert(peer_id, performance);
    }
}

/// Data structures
#[derive(Debug, Clone)]
pub struct NetworkMessage {
    pub message_type: MessageType,
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    Data,
    Control,
    Heartbeat,
    Batch,
}

#[derive(Debug)]
pub struct OptimizedMessage {
    pub original_size: usize,
    pub compressed_data: Option<Vec<u8>>,
    pub batched: bool,
    pub routing_info: Option<RoutingInfo>,
    pub peer_id: PeerId,
    pub message_type: MessageType,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct MessageBatch {
    pub messages: Vec<NetworkMessage>,
    pub target_peer: PeerId,
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct RoutingInfo {
    pub target_peer: PeerId,
    pub preferred_path: Vec<PeerId>,
    pub backup_paths: Vec<Vec<PeerId>>,
    pub estimated_latency: Duration,
    pub reliability_score: f64,
}

/// Performance-related errors
#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("Compression not beneficial")]
    CompressionNotBeneficial,
    #[error("Batch processing failed: {0}")]
    BatchProcessingFailed(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Routing optimization failed: {0}")]
    RoutingFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert!(config.enable_batching);
        assert!(config.enable_compression);
        assert!(config.enable_caching);
        assert_eq!(config.max_batch_size, 100);
    }

    #[tokio::test]
    async fn test_message_batcher() {
        let mut batcher = MessageBatcher::new(2, Duration::from_secs(1));
        let peer_id = PeerId::random();
        let message = NetworkMessage {
            message_type: MessageType::Data,
            data: vec![1, 2, 3],
            timestamp: Instant::now(),
        };

        // First message should not create a batch
        assert!(batcher.add_message(message.clone(), peer_id).await.is_none());
        
        // Second message should create a batch
        assert!(batcher.add_message(message, peer_id).await.is_some());
    }

    #[tokio::test]
    async fn test_compression_manager() {
        let mut manager = CompressionManager::new(10);
        let data = vec![0u8; 1000]; // Compressible data
        
        let compressed = manager.compress(&data).await.unwrap();
        assert!(compressed.len() < data.len());
        
        let decompressed = manager.decompress(&compressed).await.unwrap();
        assert_eq!(data, decompressed);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let mut cache = CacheManager::new(10, Duration::from_secs(60));
        let key = vec![1, 2, 3];
        let message = NetworkMessage {
            message_type: MessageType::Data,
            data: vec![4, 5, 6],
            timestamp: Instant::now(),
        };

        // Cache miss
        assert!(cache.get(&key).await.is_none());
        
        // Store in cache
        cache.put(key.clone(), message.clone()).await;
        
        // Cache hit
        assert!(cache.get(&key).await.is_some());
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.messages_batched, 0);
        assert_eq!(metrics.compression_ratio, 0.0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
    }
}
