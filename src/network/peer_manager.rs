//! Peer management for libp2p network
//!
//! Handles peer connections, scoring, and lifecycle management.

use super::{NetworkError, PeerInfo};
use crate::network::network_config::PeerConfig;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Peer connection states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    /// Connecting to peer
    Connecting,
    /// Successfully connected
    Connected,
    /// Disconnecting from peer
    Disconnecting,
    /// Disconnected
    Disconnected,
    /// Banned due to bad behavior
    Banned,
}

/// Peer scoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerScore {
    /// Base score (0-100)
    pub base_score: i32,
    /// Message validity score
    pub message_score: i32,
    /// Latency score
    pub latency_score: i32,
    /// Reliability score
    pub reliability_score: i32,
    /// Total computed score
    pub total_score: i32,
}

impl PeerScore {
    /// Create new peer score with default values
    pub fn new() -> Self {
        Self {
            base_score: 50,
            message_score: 0,
            latency_score: 0,
            reliability_score: 0,
            total_score: 50,
        }
    }
    
    /// Update total score based on components
    pub fn update_total(&mut self) {
        self.total_score = self.base_score + self.message_score + self.latency_score + self.reliability_score;
        // Clamp to valid range
        self.total_score = self.total_score.max(-100).min(100);
    }
    
    /// Apply penalty for bad behavior
    pub fn apply_penalty(&mut self, penalty: i32) {
        self.message_score -= penalty;
        self.update_total();
    }
    
    /// Apply reward for good behavior
    pub fn apply_reward(&mut self, reward: i32) {
        self.message_score += reward;
        self.update_total();
    }
}

/// Enhanced peer information with state and scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPeer {
    pub info: PeerInfo,
    pub state: PeerState,
    pub score: PeerScore,
    pub connection_attempts: u32,
    pub last_message_time: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl ManagedPeer {
    /// Create new managed peer
    pub fn new(peer_id: String, user_agent: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            info: PeerInfo {
                peer_id,
                connected_at: timestamp,
                last_seen: timestamp,
                score: 50,
                user_agent,
                protocols: Vec::new(),
            },
            state: PeerState::Connecting,
            score: PeerScore::new(),
            connection_attempts: 0,
            last_message_time: timestamp,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        }
    }
    
    /// Update peer state
    pub fn set_state(&mut self, state: PeerState) {
        let new_state = state.clone();
        self.state = state;
        if new_state == PeerState::Connected {
            self.info.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
    }
    
    /// Record message sent
    pub fn record_message_sent(&mut self, bytes: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
        self.update_last_seen();
    }
    
    /// Record message received
    pub fn record_message_received(&mut self, bytes: u64, is_valid: bool) {
        self.messages_received += 1;
        self.bytes_received += bytes;
        self.update_last_seen();
        
        if is_valid {
            self.score.apply_reward(1);
        } else {
            self.score.apply_penalty(5);
        }
        
        self.info.score = self.score.total_score;
    }
    
    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_message_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.info.last_seen = self.last_message_time;
    }
    
    /// Check if peer should be banned
    pub fn should_ban(&self) -> bool {
        self.score.total_score < -50 || self.connection_attempts > 5
    }
}

/// Connection quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionQuality {
    /// Excellent connection quality
    Excellent,
    /// Good connection quality
    Good,
    /// Fair connection quality
    Fair,
    /// Poor connection quality
    Poor,
    /// Connection quality unknown
    Unknown,
}

impl ConnectionQuality {
    /// Convert quality to numeric score (0-100)
    pub fn to_score(&self) -> u8 {
        match self {
            ConnectionQuality::Excellent => 90,
            ConnectionQuality::Good => 75,
            ConnectionQuality::Fair => 50,
            ConnectionQuality::Poor => 25,
            ConnectionQuality::Unknown => 0,
        }
    }
    
    /// Determine quality from latency and packet loss
    pub fn from_metrics(latency_ms: f64, packet_loss_rate: f64) -> Self {
        match (latency_ms, packet_loss_rate) {
            (latency, loss) if latency < 50.0 && loss < 0.01 => ConnectionQuality::Excellent,
            (latency, loss) if latency < 100.0 && loss < 0.05 => ConnectionQuality::Good,
            (latency, loss) if latency < 200.0 && loss < 0.10 => ConnectionQuality::Fair,
            _ => ConnectionQuality::Poor,
        }
    }
}

/// Connection handle for managing individual peer connections
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    /// Peer ID for this connection
    pub peer_id: String,
    /// Connection creation timestamp
    pub created_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Connection quality assessment
    pub quality: ConnectionQuality,
    /// Connection statistics
    pub stats: ConnectionStats,
    /// Whether connection is active
    pub is_active: bool,
}

impl ConnectionHandle {
    /// Create new connection handle
    pub fn new(peer_id: String) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            created_at: now,
            last_activity: now,
            quality: ConnectionQuality::Unknown,
            stats: ConnectionStats::new(),
            is_active: true,
        }
    }
    
    /// Update connection activity
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Update connection quality
    pub fn update_quality(&mut self, quality: ConnectionQuality) {
        self.quality = quality;
    }
    
    /// Check if connection is stale (inactive for too long)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
    
    /// Get connection age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Packet loss rate (0.0-1.0)
    pub packet_loss_rate: f64,
    /// Connection uptime in seconds
    pub uptime_seconds: u64,
}

impl ConnectionStats {
    /// Create new connection stats
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            avg_latency_ms: 0.0,
            packet_loss_rate: 0.0,
            uptime_seconds: 0,
        }
    }
    
    /// Update latency measurement
    pub fn update_latency(&mut self, latency_ms: f64) {
        // Exponential moving average for latency
        const ALPHA: f64 = 0.1;
        self.avg_latency_ms = ALPHA * latency_ms + (1.0 - ALPHA) * self.avg_latency_ms;
    }
    
    /// Update packet loss rate
    pub fn update_packet_loss(&mut self, loss_rate: f64) {
        // Exponential moving average for packet loss
        const ALPHA: f64 = 0.05;
        self.packet_loss_rate = ALPHA * loss_rate + (1.0 - ALPHA) * self.packet_loss_rate;
    }
    
    /// Record message sent
    pub fn record_message_sent(&mut self, bytes: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
    }
    
    /// Record message received
    pub fn record_message_received(&mut self, bytes: u64) {
        self.messages_received += 1;
        self.bytes_received += bytes;
    }
}

/// Connection request for pool management
#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    /// Peer ID to connect to
    pub peer_id: String,
    /// User agent string
    pub user_agent: String,
    /// Priority level (higher = more important)
    pub priority: u8,
    /// Request timestamp
    pub timestamp: Instant,
}

impl ConnectionRequest {
    /// Create new connection request
    pub fn new(peer_id: String, user_agent: String, priority: u8) -> Self {
        Self {
            peer_id,
            user_agent,
            priority,
            timestamp: Instant::now(),
        }
    }
}

/// Connection pool for managing multiple peer connections
#[derive(Debug)]
pub struct ConnectionPool {
    /// Active connections by peer ID
    active_connections: HashMap<String, Arc<RwLock<ConnectionHandle>>>,
    /// Connection request queue
    connection_queue: VecDeque<ConnectionRequest>,
    /// Maximum number of connections
    max_connections: usize,
    /// Connection timeout duration
    connection_timeout: Duration,
    /// Channel for connection events
    event_tx: mpsc::Sender<ConnectionEvent>,
    /// Channel receiver for connection events
    event_rx: Option<mpsc::Receiver<ConnectionEvent>>,
}

/// Connection pool events
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// Connection established
    Connected { peer_id: String },
    /// Connection failed
    Failed { peer_id: String, reason: String },
    /// Connection closed
    Closed { peer_id: String },
    /// Connection quality updated
    QualityUpdated { peer_id: String, quality: ConnectionQuality },
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(max_connections: usize, connection_timeout: Duration) -> Result<Self, NetworkError> {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        Ok(Self {
            active_connections: HashMap::new(),
            connection_queue: VecDeque::new(),
            max_connections,
            connection_timeout,
            event_tx,
            event_rx: Some(event_rx),
        })
    }
    
    /// Start connection pool background tasks
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Start connection management background task
        let event_tx = self.event_tx.clone();
        let pool = Arc::new(RwLock::new(self.active_connections.clone()));
        
        tokio::spawn(async move {
            Self::connection_management_task(event_tx, pool).await;
        });
        
        Ok(())
    }
    
    /// Background task for connection management
    async fn connection_management_task(
        event_tx: mpsc::Sender<ConnectionEvent>,
        pool: Arc<RwLock<HashMap<String, Arc<RwLock<ConnectionHandle>>>>>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Clean up stale connections
            let mut pool_guard = pool.write().await;
            let stale_connections: Vec<String> = pool_guard
                .iter()
                .filter(|(_, handle)| {
                    if let Ok(handle_guard) = handle.read() {
                        handle_guard.is_stale(Duration::from_secs(300)) // 5 minutes
                    } else {
                        false
                    }
                })
                .map(|(peer_id, _)| peer_id.clone())
                .collect();
            
            for peer_id in stale_connections {
                if let Some(handle) = pool_guard.remove(&peer_id) {
                    if let Ok(mut handle_guard) = handle.write() {
                        handle_guard.is_active = false;
                    }
                    
                    let _ = event_tx.send(ConnectionEvent::Closed { peer_id }).await;
                }
            }
        }
    }
    
    /// Establish connection to peer
    pub async fn establish_connection(&mut self, peer_id: String, user_agent: String) -> Result<Arc<RwLock<ConnectionHandle>>, NetworkError> {
        // Check if connection already exists
        if let Some(handle) = self.active_connections.get(&peer_id) {
            if let Ok(handle_guard) = handle.read() {
                if handle_guard.is_active {
                    return Ok(handle.clone());
                }
            }
        }
        
        // Check connection limit
        if self.active_connections.len() >= self.max_connections {
            return Err(NetworkError::Configuration { 
                details: "Connection pool is full".to_string() 
            });
        }
        
        // Simulate async connection establishment
        // In real implementation, this would use libp2p
        let connection_result = self.simulate_connection_establishment(&peer_id).await;
        
        match connection_result {
            Ok(handle) => {
                let handle_arc = Arc::new(RwLock::new(handle));
                self.active_connections.insert(peer_id.clone(), handle_arc.clone());
                
                // Send connection event
                let _ = self.event_tx.send(ConnectionEvent::Connected { 
                    peer_id: peer_id.clone() 
                }).await;
                
                Ok(handle_arc)
            }
            Err(e) => {
                // Send failure event
                let _ = self.event_tx.send(ConnectionEvent::Failed { 
                    peer_id: peer_id.clone(),
                    reason: e.to_string(),
                }).await;
                
                Err(e)
            }
        }
    }
    
    /// Simulate connection establishment (placeholder for libp2p integration)
    async fn simulate_connection_establishment(&self, peer_id: &str) -> Result<ConnectionHandle, NetworkError> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Simulate connection success/failure based on peer_id
        if peer_id.contains("bad") {
            return Err(NetworkError::ConnectionFailed { 
                peer_id: peer_id.to_string() 
            });
        }
        
        let mut handle = ConnectionHandle::new(peer_id.to_string());
        
        // Simulate initial quality assessment
        handle.update_quality(ConnectionQuality::Good);
        
        Ok(handle)
    }
    
    /// Close connection to peer
    pub async fn close_connection(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        if let Some(handle) = self.active_connections.remove(peer_id) {
            if let Ok(mut handle_guard) = handle.write() {
                handle_guard.is_active = false;
            }
            
            // Send connection closed event
            let _ = self.event_tx.send(ConnectionEvent::Closed { 
                peer_id: peer_id.to_string() 
            }).await;
            
            Ok(())
        } else {
            Err(NetworkError::ConnectionFailed { 
                peer_id: peer_id.to_string() 
            })
        }
    }
    
    /// Get connection handle for peer
    pub fn get_connection_handle(&self, peer_id: &str) -> Option<Arc<RwLock<ConnectionHandle>>> {
        self.active_connections.get(peer_id).cloned()
    }
    
    /// Get connection quality for peer
    pub async fn get_connection_quality(&self, peer_id: &str) -> Option<ConnectionQuality> {
        if let Some(handle) = self.active_connections.get(peer_id) {
            if let Ok(handle_guard) = handle.read() {
                return Some(handle_guard.quality.clone());
            }
        }
        None
    }
    
    /// Update connection quality for peer
    pub async fn update_connection_quality(&self, peer_id: &str, quality: ConnectionQuality) -> Result<(), NetworkError> {
        if let Some(handle) = self.active_connections.get(peer_id) {
            if let Ok(mut handle_guard) = handle.write() {
                handle_guard.update_quality(quality.clone());
                
                // Send quality update event
                let _ = self.event_tx.send(ConnectionEvent::QualityUpdated { 
                    peer_id: peer_id.to_string(),
                    quality,
                }).await;
                
                return Ok(());
            }
        }
        
        Err(NetworkError::ConnectionFailed { 
            peer_id: peer_id.to_string() 
        })
    }
    
    /// Get connection statistics
    pub async fn get_connection_stats(&self, peer_id: &str) -> Option<ConnectionStats> {
        if let Some(handle) = self.active_connections.get(peer_id) {
            if let Ok(handle_guard) = handle.read() {
                return Some(handle_guard.stats.clone());
            }
        }
        None
    }
    
    /// Get pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        let active_count = self.active_connections.len();
        let queue_size = self.connection_queue.len();
        
        PoolStats {
            active_connections: active_count,
            queued_connections: queue_size,
            max_connections: self.max_connections,
            pool_utilization: (active_count as f64 / self.max_connections as f64) * 100.0,
        }
    }
    
    /// Add connection request to queue
    pub fn queue_connection(&mut self, request: ConnectionRequest) -> Result<(), NetworkError> {
        if self.connection_queue.len() >= 1000 {
            return Err(NetworkError::Configuration { 
                details: "Connection queue is full".to_string() 
            });
        }
        
        self.connection_queue.push_back(request);
        Ok(())
    }
    
    /// Process connection queue
    pub async fn process_connection_queue(&mut self) -> Result<(), NetworkError> {
        while let Some(request) = self.connection_queue.pop_front() {
            // Check if we have capacity
            if self.active_connections.len() >= self.max_connections {
                // Put request back in queue
                self.connection_queue.push_front(request);
                break;
            }
            
            // Try to establish connection
            let _ = self.establish_connection(request.peer_id, request.user_agent).await;
        }
        
        Ok(())
    }
    
    /// Get next connection event
    pub async fn next_event(&mut self) -> Option<ConnectionEvent> {
        if let Some(rx) = &mut self.event_rx {
            rx.recv().await
        } else {
            None
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Number of queued connections
    pub queued_connections: usize,
    /// Maximum connections allowed
    pub max_connections: usize,
    /// Pool utilization percentage
    pub pool_utilization: f64,
}

/// Peer manager for network connections
#[derive(Debug)]
pub struct PeerManager {
    /// Configuration for peer management
    config: PeerConfig,
    /// Active peers by peer ID
    peers: HashMap<String, ManagedPeer>,
    /// Banned peers list
    banned_peers: HashMap<String, u64>, // peer_id -> ban_until_timestamp
    /// Connection limits
    max_peers: u32,
    target_peers: u32,
    /// Connection pool for managing libp2p connections
    connection_pool: Option<ConnectionPool>,
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(config: PeerConfig) -> Result<Self, NetworkError> {
        let connection_pool = ConnectionPool::new(config.max_peers as usize, Duration::from_secs(30));
        connection_pool.map(|mut pool| {
            pool.start().unwrap(); // Start the background task
            Self {
                max_peers: config.max_peers,
                target_peers: config.target_peers,
                config,
                peers: HashMap::new(),
                banned_peers: HashMap::new(),
                connection_pool: Some(pool),
            }
        })
    }
    
    /// Start peer manager
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Initialize peer discovery and connection logic
        Ok(())
    }
    
    /// Establish connection to peer using connection pool
    pub async fn establish_connection(&mut self, peer_id: String, user_agent: String) -> Result<(), NetworkError> {
        if let Some(pool) = &mut self.connection_pool {
            let _handle = pool.establish_connection(peer_id.clone(), user_agent).await?;
            
            // Add to peer management
            self.add_peer(peer_id, user_agent)?;
            
            Ok(())
        } else {
            Err(NetworkError::Configuration { 
                details: "Connection pool not initialized".to_string() 
            })
        }
    }
    
    /// Close connection to peer using connection pool
    pub async fn close_connection(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        if let Some(pool) = &mut self.connection_pool {
            pool.close_connection(peer_id).await?;
        }
        
        // Remove from peer management
        self.remove_peer(peer_id);
        
        Ok(())
    }
    
    /// Get connection quality for peer
    pub async fn get_connection_quality(&self, peer_id: &str) -> Option<ConnectionQuality> {
        if let Some(pool) = &self.connection_pool {
            pool.get_connection_quality(peer_id).await
        } else {
            None
        }
    }
    
    /// Update connection quality for peer
    pub async fn update_connection_quality(&self, peer_id: &str, quality: ConnectionQuality) -> Result<(), NetworkError> {
        if let Some(pool) = &self.connection_pool {
            pool.update_connection_quality(peer_id, quality).await
        } else {
            Err(NetworkError::Configuration { 
                details: "Connection pool not initialized".to_string() 
            })
        }
    }
    
    /// Get connection statistics for peer
    pub async fn get_connection_stats(&self, peer_id: &str) -> Option<ConnectionStats> {
        if let Some(pool) = &self.connection_pool {
            pool.get_connection_stats(peer_id).await
        } else {
            None
        }
    }
    
    /// Get pool statistics
    pub fn get_pool_stats(&self) -> Option<PoolStats> {
        self.connection_pool.as_ref().map(|pool| pool.get_pool_stats())
    }
    
    /// Queue connection request
    pub fn queue_connection(&mut self, peer_id: String, user_agent: String, priority: u8) -> Result<(), NetworkError> {
        if let Some(pool) = &mut self.connection_pool {
            let request = ConnectionRequest::new(peer_id, user_agent, priority);
            pool.queue_connection(request)
        } else {
            Err(NetworkError::Configuration { 
                details: "Connection pool not initialized".to_string() 
            })
        }
    }
    
    /// Process connection queue
    pub async fn process_connection_queue(&mut self) -> Result<(), NetworkError> {
        if let Some(pool) = &mut self.connection_pool {
            pool.process_connection_queue().await
        } else {
            Err(NetworkError::Configuration { 
                details: "Connection pool not initialized".to_string() 
            })
        }
    }
    
    /// Get next connection event
    pub async fn next_connection_event(&mut self) -> Option<ConnectionEvent> {
        if let Some(pool) = &mut self.connection_pool {
            pool.next_event().await
        } else {
            None
        }
    }
    
    /// Get connection handle for peer
    pub fn get_connection_handle(&self, peer_id: &str) -> Option<Arc<RwLock<ConnectionHandle>>> {
        if let Some(pool) = &self.connection_pool {
            pool.get_connection_handle(peer_id)
        } else {
            None
        }
    }
    
    /// Add new peer
    pub fn add_peer(&mut self, peer_id: String, user_agent: String) -> Result<(), NetworkError> {
        if self.is_banned(&peer_id) {
            return Err(NetworkError::ConnectionFailed { 
                peer_id: peer_id.clone() 
            });
        }
        
        if self.peers.len() >= self.max_peers as usize {
            return Err(NetworkError::Configuration { 
                details: "Maximum peer limit reached".to_string() 
            });
        }
        
        let peer = ManagedPeer::new(peer_id.clone(), user_agent);
        self.peers.insert(peer_id, peer);
        Ok(())
    }
    
    /// Remove peer
    pub fn remove_peer(&mut self, peer_id: &str) -> Option<ManagedPeer> {
        self.peers.remove(peer_id)
    }
    
    /// Get peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Option<&ManagedPeer> {
        self.peers.get(peer_id)
    }
    
    /// Get mutable peer by ID
    pub fn get_peer_mut(&mut self, peer_id: &str) -> Option<&mut ManagedPeer> {
        self.peers.get_mut(peer_id)
    }
    
    /// Get total peer count
    pub fn get_peer_count(&self) -> u32 {
        self.peers.len() as u32
    }
    
    /// Get connected peer count
    pub fn get_connected_count(&self) -> u32 {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected)
            .count() as u32
    }
    
    /// Ban peer for specified duration
    pub fn ban_peer(&mut self, peer_id: &str, duration_secs: u64) {
        let ban_until = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() + duration_secs;
            
        self.banned_peers.insert(peer_id.to_string(), ban_until);
        self.remove_peer(peer_id);
    }
    
    /// Check if peer is banned
    pub fn is_banned(&self, peer_id: &str) -> bool {
        if let Some(&ban_until) = self.banned_peers.get(peer_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            ban_until > now
        } else {
            false
        }
    }
    
    /// Cleanup expired bans and inactive peers
    pub fn cleanup(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Remove expired bans
        self.banned_peers.retain(|_, &mut ban_until| ban_until > now);
        
        // Check for peers that should be banned
        let peers_to_ban: Vec<String> = self.peers.iter()
            .filter(|(_, peer)| peer.should_ban())
            .map(|(id, _)| id.clone())
            .collect();
            
        for peer_id in peers_to_ban {
            self.ban_peer(&peer_id, 3600); // Ban for 1 hour
        }
        
        // Remove inactive peers (no activity for 5 minutes)
        let inactive_threshold = now - 300;
        self.peers.retain(|_, peer| peer.last_message_time > inactive_threshold);
    }
    
    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<&ManagedPeer> {
        self.peers.values()
            .filter(|p| p.state == PeerState::Connected)
            .collect()
    }
    
    /// Get peer statistics
    pub fn get_peer_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_peers".to_string(), 
            serde_json::Value::Number(self.peers.len().into()));
        stats.insert("connected_peers".to_string(), 
            serde_json::Value::Number(self.get_connected_count().into()));
        stats.insert("banned_peers".to_string(), 
            serde_json::Value::Number(self.banned_peers.len().into()));
        stats.insert("target_peers".to_string(), 
            serde_json::Value::Number(self.target_peers.into()));
        stats.insert("max_peers".to_string(), 
            serde_json::Value::Number(self.max_peers.into()));
            
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::network_config::PeerConfig;
    
    #[test]
    fn test_peer_score_creation() {
        let score = PeerScore::new();
        assert_eq!(score.base_score, 50);
        assert_eq!(score.total_score, 50);
    }
    
    #[test]
    fn test_peer_score_penalty() {
        let mut score = PeerScore::new();
        score.apply_penalty(10);
        assert_eq!(score.message_score, -10);
        assert_eq!(score.total_score, 40);
    }
    
    #[test]
    fn test_peer_score_reward() {
        let mut score = PeerScore::new();
        score.apply_reward(20);
        assert_eq!(score.message_score, 20);
        assert_eq!(score.total_score, 70);
    }
    
    #[test]
    fn test_managed_peer_creation() {
        let peer = ManagedPeer::new("test_peer".to_string(), "panro/1.0".to_string());
        assert_eq!(peer.info.peer_id, "test_peer");
        assert_eq!(peer.info.user_agent, "panro/1.0");
        assert_eq!(peer.state, PeerState::Connecting);
        assert_eq!(peer.score.total_score, 50);
    }
    
    #[test]
    fn test_peer_manager_creation() {
        let config = PeerConfig::default();
        let manager = PeerManager::new(config);
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert_eq!(manager.get_peer_count(), 0);
    }
    
    #[test]
    fn test_peer_manager_add_peer() {
        let config = PeerConfig::default();
        let mut manager = PeerManager::new(config).unwrap();
        
        let result = manager.add_peer("test_peer".to_string(), "panro/1.0".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_peer_count(), 1);
        
        let peer = manager.get_peer("test_peer");
        assert!(peer.is_some());
        assert_eq!(peer.unwrap().info.peer_id, "test_peer");
    }
    
    #[test]
    fn test_peer_banning() {
        let config = PeerConfig::default();
        let mut manager = PeerManager::new(config).unwrap();
        
        // Add peer first
        manager.add_peer("bad_peer".to_string(), "test".to_string()).unwrap();
        assert_eq!(manager.get_peer_count(), 1);
        
        // Ban the peer
        manager.ban_peer("bad_peer", 3600);
        assert_eq!(manager.get_peer_count(), 0);
        assert!(manager.is_banned("bad_peer"));
        
        // Try to add banned peer again
        let result = manager.add_peer("bad_peer".to_string(), "test".to_string());
        assert!(result.is_err());
    }
    
    // Connection Quality Tests
    #[test]
    fn test_connection_quality_scoring() {
        assert_eq!(ConnectionQuality::Excellent.to_score(), 90);
        assert_eq!(ConnectionQuality::Good.to_score(), 75);
        assert_eq!(ConnectionQuality::Fair.to_score(), 50);
        assert_eq!(ConnectionQuality::Poor.to_score(), 25);
        assert_eq!(ConnectionQuality::Unknown.to_score(), 0);
    }
    
    #[test]
    fn test_connection_quality_from_metrics() {
        // Excellent connection
        let quality = ConnectionQuality::from_metrics(30.0, 0.005);
        assert!(matches!(quality, ConnectionQuality::Excellent));
        
        // Good connection
        let quality = ConnectionQuality::from_metrics(80.0, 0.03);
        assert!(matches!(quality, ConnectionQuality::Good));
        
        // Fair connection
        let quality = ConnectionQuality::from_metrics(150.0, 0.08);
        assert!(matches!(quality, ConnectionQuality::Fair));
        
        // Poor connection
        let quality = ConnectionQuality::from_metrics(300.0, 0.15);
        assert!(matches!(quality, ConnectionQuality::Poor));
    }
    
    // Connection Handle Tests
    #[test]
    fn test_connection_handle_creation() {
        let handle = ConnectionHandle::new("test_peer".to_string());
        assert_eq!(handle.peer_id, "test_peer");
        assert!(handle.is_active);
        assert!(matches!(handle.quality, ConnectionQuality::Unknown));
    }
    
    #[test]
    fn test_connection_handle_activity() {
        let mut handle = ConnectionHandle::new("test_peer".to_string());
        let initial_activity = handle.last_activity;
        
        std::thread::sleep(Duration::from_millis(10));
        handle.update_activity();
        
        assert!(handle.last_activity > initial_activity);
    }
    
    #[test]
    fn test_connection_handle_stale_detection() {
        let handle = ConnectionHandle::new("test_peer".to_string());
        assert!(!handle.is_stale(Duration::from_secs(1)));
        
        // Note: Testing stale detection with real time would be flaky
        // In real implementation, we'd use mocked time
    }
    
    // Connection Stats Tests
    #[test]
    fn test_connection_stats_creation() {
        let stats = ConnectionStats::new();
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.avg_latency_ms, 0.0);
        assert_eq!(stats.packet_loss_rate, 0.0);
    }
    
    #[test]
    fn test_connection_stats_updates() {
        let mut stats = ConnectionStats::new();
        
        // Test message recording
        stats.record_message_sent(100);
        stats.record_message_received(200);
        
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_sent, 100);
        assert_eq!(stats.bytes_received, 200);
        
        // Test latency update
        stats.update_latency(50.0);
        assert!(stats.avg_latency_ms > 0.0);
        
        // Test packet loss update
        stats.update_packet_loss(0.05);
        assert!(stats.packet_loss_rate > 0.0);
    }
    
    // Connection Request Tests
    #[test]
    fn test_connection_request_creation() {
        let request = ConnectionRequest::new(
            "test_peer".to_string(),
            "panro/1.0".to_string(),
            5
        );
        
        assert_eq!(request.peer_id, "test_peer");
        assert_eq!(request.user_agent, "panro/1.0");
        assert_eq!(request.priority, 5);
    }
    
    // Connection Pool Tests
    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool = ConnectionPool::new(10, Duration::from_secs(30));
        assert!(pool.is_ok());
        
        let mut pool = pool.unwrap();
        let stats = pool.get_pool_stats();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.max_connections, 10);
    }
    
    #[tokio::test]
    async fn test_connection_pool_establishment() {
        let mut pool = ConnectionPool::new(5, Duration::from_secs(30)).unwrap();
        
        // Establish connection
        let result = pool.establish_connection("good_peer".to_string(), "test".to_string()).await;
        assert!(result.is_ok());
        
        let stats = pool.get_pool_stats();
        assert_eq!(stats.active_connections, 1);
    }
    
    #[tokio::test]
    async fn test_connection_pool_limit() {
        let mut pool = ConnectionPool::new(2, Duration::from_secs(30)).unwrap();
        
        // Add connections up to limit
        assert!(pool.establish_connection("peer1".to_string(), "test".to_string()).await.is_ok());
        assert!(pool.establish_connection("peer2".to_string(), "test".to_string()).await.is_ok());
        
        // Try to exceed limit
        let result = pool.establish_connection("peer3".to_string(), "test".to_string()).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_connection_pool_quality_management() {
        let mut pool = ConnectionPool::new(5, Duration::from_secs(30)).unwrap();
        
        // Establish connection
        pool.establish_connection("test_peer".to_string(), "test".to_string()).await.unwrap();
        
        // Update quality
        let result = pool.update_connection_quality("test_peer", ConnectionQuality::Excellent).await;
        assert!(result.is_ok());
        
        // Get quality
        let quality = pool.get_connection_quality("test_peer").await;
        assert!(quality.is_some());
        assert!(matches!(quality.unwrap(), ConnectionQuality::Excellent));
    }
    
    #[tokio::test]
    async fn test_connection_pool_queue_management() {
        let mut pool = ConnectionPool::new(1, Duration::from_secs(30)).unwrap();
        
        // Fill the pool
        pool.establish_connection("peer1".to_string(), "test".to_string()).await.unwrap();
        
        // Queue additional connection
        let request = ConnectionRequest::new("peer2".to_string(), "test".to_string(), 1);
        assert!(pool.queue_connection(request).is_ok());
        
        let stats = pool.get_pool_stats();
        assert_eq!(stats.queued_connections, 1);
    }
    
    #[tokio::test]
    async fn test_connection_pool_events() {
        let mut pool = ConnectionPool::new(5, Duration::from_secs(30)).unwrap();
        
        // Establish connection and check for event
        pool.establish_connection("test_peer".to_string(), "test".to_string()).await.unwrap();
        
        // Get next event
        let event = pool.next_event().await;
        assert!(event.is_some());
        
        if let Some(ConnectionEvent::Connected { peer_id }) = event {
            assert_eq!(peer_id, "test_peer");
        } else {
            panic!("Expected Connected event");
        }
    }
    
    // PeerManager Integration Tests
    #[tokio::test]
    async fn test_peer_manager_connection_integration() {
        let config = PeerConfig::default();
        let mut manager = PeerManager::new(config).unwrap();
        
        // Establish connection through peer manager
        let result = manager.establish_connection("test_peer".to_string(), "panro/1.0".to_string()).await;
        assert!(result.is_ok());
        
        // Check that peer was added
        assert_eq!(manager.get_peer_count(), 1);
        
        // Check pool stats
        let pool_stats = manager.get_pool_stats();
        assert!(pool_stats.is_some());
        assert_eq!(pool_stats.unwrap().active_connections, 1);
    }
    
    #[tokio::test]
    async fn test_peer_manager_connection_quality() {
        let config = PeerConfig::default();
        let manager = PeerManager::new(config).unwrap();
        
        // Establish connection
        manager.establish_connection("test_peer".to_string(), "panro/1.0".to_string()).await.unwrap();
        
        // Update quality
        let result = manager.update_connection_quality("test_peer", ConnectionQuality::Good).await;
        assert!(result.is_ok());
        
        // Get quality
        let quality = manager.get_connection_quality("test_peer").await;
        assert!(quality.is_some());
        assert!(matches!(quality.unwrap(), ConnectionQuality::Good));
    }
    
    #[tokio::test]
    async fn test_peer_manager_connection_stats() {
        let config = PeerConfig::default();
        let manager = PeerManager::new(config).unwrap();
        
        // Establish connection
        manager.establish_connection("test_peer".to_string(), "panro/1.0".to_string()).await.unwrap();
        
        // Get stats
        let stats = manager.get_connection_stats("test_peer").await;
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.messages_received, 0);
    }
}
