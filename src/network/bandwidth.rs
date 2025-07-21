//! Bandwidth management and optimization module
//!
//! Provides bandwidth monitoring, rate limiting, and optimization
//! for P2P network communications.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Bandwidth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthConfig {
    /// Maximum upload bandwidth in bytes per second
    pub max_upload_bps: u64,
    /// Maximum download bandwidth in bytes per second
    pub max_download_bps: u64,
    /// Rate limiting window in seconds
    pub rate_window_secs: u64,
    /// Enable bandwidth monitoring
    pub enable_monitoring: bool,
    /// Burst allowance multiplier
    pub burst_multiplier: f64,
}

impl Default for BandwidthConfig {
    fn default() -> Self {
        Self {
            max_upload_bps: 10_000_000,   // 10 MB/s
            max_download_bps: 50_000_000, // 50 MB/s
            rate_window_secs: 60,
            enable_monitoring: true,
            burst_multiplier: 1.5,
        }
    }
}

/// Bandwidth statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub current_upload_bps: u64,
    pub current_download_bps: u64,
    pub peak_upload_bps: u64,
    pub peak_download_bps: u64,
    pub total_connections: usize,
    pub active_connections: usize,
    pub last_reset: Instant,
}

impl Default for BandwidthStats {
    fn default() -> Self {
        Self {
            total_bytes_sent: 0,
            total_bytes_received: 0,
            current_upload_bps: 0,
            current_download_bps: 0,
            peak_upload_bps: 0,
            peak_download_bps: 0,
            total_connections: 0,
            active_connections: 0,
            last_reset: Instant::now(),
        }
    }
}

/// Per-peer bandwidth tracking
#[derive(Debug, Clone)]
pub struct PeerBandwidth {
    pub peer_id: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_activity: Instant,
    pub upload_rate: f64,
    pub download_rate: f64,
}

/// Bandwidth limiter with token bucket algorithm
pub struct BandwidthLimiter {
    config: BandwidthConfig,
    stats: Arc<RwLock<BandwidthStats>>,
    peer_stats: Arc<RwLock<HashMap<String, PeerBandwidth>>>,
    upload_tokens: Arc<RwLock<f64>>,
    download_tokens: Arc<RwLock<f64>>,
    last_refill: Arc<RwLock<Instant>>,
}

impl BandwidthLimiter {
    /// Create new bandwidth limiter
    pub fn new(config: BandwidthConfig) -> Self {
        Self {
            upload_tokens: Arc::new(RwLock::new(config.max_upload_bps as f64)),
            download_tokens: Arc::new(RwLock::new(config.max_download_bps as f64)),
            last_refill: Arc::new(RwLock::new(Instant::now())),
            stats: Arc::new(RwLock::new(BandwidthStats::default())),
            peer_stats: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Check if upload is allowed for given bytes
    pub async fn can_upload(&self, bytes: u64) -> bool {
        self.refill_tokens().await;
        
        let mut tokens = self.upload_tokens.write().await;
        if *tokens >= bytes as f64 {
            *tokens -= bytes as f64;
            true
        } else {
            false
        }
    }
    
    /// Check if download is allowed for given bytes
    pub async fn can_download(&self, bytes: u64) -> bool {
        self.refill_tokens().await;
        
        let mut tokens = self.download_tokens.write().await;
        if *tokens >= bytes as f64 {
            *tokens -= bytes as f64;
            true
        } else {
            false
        }
    }
    
    /// Record uploaded bytes
    pub async fn record_upload(&self, peer_id: &str, bytes: u64) {
        // Update global stats
        {
            let mut stats = self.stats.write().await;
            stats.total_bytes_sent += bytes;
        }
        
        // Update peer stats
        {
            let mut peer_stats = self.peer_stats.write().await;
            let peer_bandwidth = peer_stats.entry(peer_id.to_string())
                .or_insert_with(|| PeerBandwidth {
                    peer_id: peer_id.to_string(),
                    bytes_sent: 0,
                    bytes_received: 0,
                    last_activity: Instant::now(),
                    upload_rate: 0.0,
                    download_rate: 0.0,
                });
            
            peer_bandwidth.bytes_sent += bytes;
            peer_bandwidth.last_activity = Instant::now();
        }
        
        self.update_current_rates().await;
    }
    
    /// Record downloaded bytes
    pub async fn record_download(&self, peer_id: &str, bytes: u64) {
        // Update global stats
        {
            let mut stats = self.stats.write().await;
            stats.total_bytes_received += bytes;
        }
        
        // Update peer stats
        {
            let mut peer_stats = self.peer_stats.write().await;
            let peer_bandwidth = peer_stats.entry(peer_id.to_string())
                .or_insert_with(|| PeerBandwidth {
                    peer_id: peer_id.to_string(),
                    bytes_sent: 0,
                    bytes_received: 0,
                    last_activity: Instant::now(),
                    upload_rate: 0.0,
                    download_rate: 0.0,
                });
            
            peer_bandwidth.bytes_received += bytes;
            peer_bandwidth.last_activity = Instant::now();
        }
        
        self.update_current_rates().await;
    }
    
    /// Get current bandwidth statistics
    pub async fn get_stats(&self) -> BandwidthStats {
        self.stats.read().await.clone()
    }
    
    /// Get peer bandwidth statistics
    pub async fn get_peer_stats(&self) -> Vec<PeerBandwidth> {
        let peer_stats = self.peer_stats.read().await;
        peer_stats.values().cloned().collect()
    }
    
    /// Reset bandwidth statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = BandwidthStats::default();
        
        let mut peer_stats = self.peer_stats.write().await;
        peer_stats.clear();
    }
    
    /// Cleanup inactive peers
    pub async fn cleanup_inactive_peers(&self, max_idle: Duration) {
        let mut peer_stats = self.peer_stats.write().await;
        let now = Instant::now();
        
        peer_stats.retain(|_, peer| {
            now.duration_since(peer.last_activity) < max_idle
        });
    }
    
    /// Get top bandwidth consumers
    pub async fn get_top_consumers(&self, limit: usize) -> Vec<PeerBandwidth> {
        let peer_stats = self.peer_stats.read().await;
        let mut peers: Vec<PeerBandwidth> = peer_stats.values().cloned().collect();
        
        // Sort by total bandwidth usage (sent + received)
        peers.sort_by(|a, b| {
            let a_total = a.bytes_sent + a.bytes_received;
            let b_total = b.bytes_sent + b.bytes_received;
            b_total.cmp(&a_total)
        });
        
        peers.into_iter().take(limit).collect()
    }
    
    /// Apply rate limiting policies
    pub async fn apply_rate_limiting(&self, peer_id: &str) -> RateLimitAction {
        let peer_stats = self.peer_stats.read().await;
        
        if let Some(peer) = peer_stats.get(peer_id) {
            // Check if peer is consuming too much bandwidth
            let total_rate = peer.upload_rate + peer.download_rate;
            let max_per_peer = (self.config.max_upload_bps + self.config.max_download_bps) as f64 * 0.1; // 10% max per peer
            
            if total_rate > max_per_peer {
                return RateLimitAction::Throttle(0.5); // Reduce to 50%
            }
            
            // Check if peer is idle
            if peer.last_activity.elapsed() > Duration::from_secs(300) {
                return RateLimitAction::Disconnect;
            }
        }
        
        RateLimitAction::Allow
    }
    
    // Helper methods
    
    async fn refill_tokens(&self) {
        let mut last_refill = self.last_refill.write().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        
        if elapsed > 0.0 {
            // Refill upload tokens
            {
                let mut upload_tokens = self.upload_tokens.write().await;
                let refill_amount = self.config.max_upload_bps as f64 * elapsed;
                let max_capacity = self.config.max_upload_bps as f64 * self.config.burst_multiplier;
                *upload_tokens = (*upload_tokens + refill_amount).min(max_capacity);
            }
            
            // Refill download tokens
            {
                let mut download_tokens = self.download_tokens.write().await;
                let refill_amount = self.config.max_download_bps as f64 * elapsed;
                let max_capacity = self.config.max_download_bps as f64 * self.config.burst_multiplier;
                *download_tokens = (*download_tokens + refill_amount).min(max_capacity);
            }
            
            *last_refill = now;
        }
    }
    
    async fn update_current_rates(&self) {
        if !self.config.enable_monitoring {
            return;
        }
        
        let now = Instant::now();
        let window = Duration::from_secs(self.config.rate_window_secs);
        
        // Update global rates
        {
            let mut stats = self.stats.write().await;
            let elapsed = now.duration_since(stats.last_reset).as_secs_f64();
            
            if elapsed >= self.config.rate_window_secs as f64 {
                // Reset counters for rate calculation
                stats.current_upload_bps = (stats.total_bytes_sent as f64 / elapsed) as u64;
                stats.current_download_bps = (stats.total_bytes_received as f64 / elapsed) as u64;
                
                // Update peaks
                stats.peak_upload_bps = stats.peak_upload_bps.max(stats.current_upload_bps);
                stats.peak_download_bps = stats.peak_download_bps.max(stats.current_download_bps);
                
                stats.last_reset = now;
            }
        }
        
        // Update peer rates
        {
            let mut peer_stats = self.peer_stats.write().await;
            for peer in peer_stats.values_mut() {
                let elapsed = now.duration_since(peer.last_activity).as_secs_f64().max(1.0);
                peer.upload_rate = peer.bytes_sent as f64 / elapsed;
                peer.download_rate = peer.bytes_received as f64 / elapsed;
            }
        }
    }
}

/// Rate limiting action
#[derive(Debug, Clone)]
pub enum RateLimitAction {
    Allow,
    Throttle(f64), // Throttle to this fraction of original rate
    Disconnect,
}

/// Bandwidth monitor for real-time tracking
pub struct BandwidthMonitor {
    limiter: Arc<BandwidthLimiter>,
    monitoring_interval: Duration,
}

impl BandwidthMonitor {
    /// Create new bandwidth monitor
    pub fn new(limiter: Arc<BandwidthLimiter>, monitoring_interval: Duration) -> Self {
        Self {
            limiter,
            monitoring_interval,
        }
    }
    
    /// Start monitoring bandwidth usage
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(self.monitoring_interval);
        
        loop {
            interval.tick().await;
            
            // Update rates
            self.limiter.update_current_rates().await;
            
            // Cleanup inactive peers
            self.limiter.cleanup_inactive_peers(Duration::from_secs(600)).await;
            
            // Log bandwidth statistics
            let stats = self.limiter.get_stats().await;
            tracing::debug!(
                "Bandwidth stats - Upload: {} B/s, Download: {} B/s, Connections: {}",
                stats.current_upload_bps,
                stats.current_download_bps,
                stats.active_connections
            );
        }
    }
    
    /// Generate bandwidth report
    pub async fn generate_report(&self) -> BandwidthReport {
        let stats = self.limiter.get_stats().await;
        let peer_stats = self.limiter.get_peer_stats().await;
        let top_consumers = self.limiter.get_top_consumers(10).await;
        
        BandwidthReport {
            global_stats: stats,
            peer_count: peer_stats.len(),
            top_consumers,
            timestamp: Instant::now(),
        }
    }
}

/// Bandwidth usage report
#[derive(Debug, Clone)]
pub struct BandwidthReport {
    pub global_stats: BandwidthStats,
    pub peer_count: usize,
    pub top_consumers: Vec<PeerBandwidth>,
    pub timestamp: Instant,
}

impl BandwidthReport {
    /// Print human-readable report
    pub fn print_report(&self) {
        println!("\n📊 BANDWIDTH USAGE REPORT");
        println!("========================");
        println!("Total Sent: {} MB", self.global_stats.total_bytes_sent / 1024 / 1024);
        println!("Total Received: {} MB", self.global_stats.total_bytes_received / 1024 / 1024);
        println!("Current Upload: {} KB/s", self.global_stats.current_upload_bps / 1024);
        println!("Current Download: {} KB/s", self.global_stats.current_download_bps / 1024);
        println!("Peak Upload: {} KB/s", self.global_stats.peak_upload_bps / 1024);
        println!("Peak Download: {} KB/s", self.global_stats.peak_download_bps / 1024);
        println!("Active Peers: {}", self.peer_count);
        
        if !self.top_consumers.is_empty() {
            println!("\nTop Bandwidth Consumers:");
            for (i, peer) in self.top_consumers.iter().enumerate() {
                println!("  {}. {} - Sent: {} KB, Received: {} KB", 
                        i + 1,
                        &peer.peer_id[..8],
                        peer.bytes_sent / 1024,
                        peer.bytes_received / 1024);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_bandwidth_limiter() {
        let config = BandwidthConfig {
            max_upload_bps: 1000,
            max_download_bps: 2000,
            rate_window_secs: 1,
            enable_monitoring: true,
            burst_multiplier: 1.5,
        };
        
        let limiter = BandwidthLimiter::new(config);
        
        // Test upload limiting
        assert!(limiter.can_upload(500).await);
        assert!(limiter.can_upload(400).await);
        assert!(!limiter.can_upload(200).await); // Should exceed limit
        
        // Wait for token refill
        sleep(Duration::from_millis(1100)).await;
        
        // Should be able to upload again
        assert!(limiter.can_upload(500).await);
    }
    
    #[tokio::test]
    async fn test_bandwidth_recording() {
        let config = BandwidthConfig::default();
        let limiter = BandwidthLimiter::new(config);
        
        // Record some traffic
        limiter.record_upload("peer1", 1000).await;
        limiter.record_download("peer1", 2000).await;
        limiter.record_upload("peer2", 500).await;
        
        let stats = limiter.get_stats().await;
        assert_eq!(stats.total_bytes_sent, 1500);
        assert_eq!(stats.total_bytes_received, 2000);
        
        let peer_stats = limiter.get_peer_stats().await;
        assert_eq!(peer_stats.len(), 2);
    }
    
    #[tokio::test]
    async fn test_top_consumers() {
        let config = BandwidthConfig::default();
        let limiter = BandwidthLimiter::new(config);
        
        // Create different usage patterns
        limiter.record_upload("peer1", 1000).await;
        limiter.record_download("peer1", 500).await;
        
        limiter.record_upload("peer2", 2000).await;
        limiter.record_download("peer2", 1000).await;
        
        limiter.record_upload("peer3", 500).await;
        
        let top_consumers = limiter.get_top_consumers(2).await;
        assert_eq!(top_consumers.len(), 2);
        assert_eq!(top_consumers[0].peer_id, "peer2"); // Highest total usage
        assert_eq!(top_consumers[1].peer_id, "peer1");
    }
}
