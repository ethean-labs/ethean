//! Peer management for libp2p network
//!
//! Handles peer connections, scoring, and lifecycle management.

use super::{NetworkError, PeerInfo};
use crate::network::network_config::PeerConfig;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
            self.info.last_seen = Instant::now();
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
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(config: PeerConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            max_peers: config.max_peers,
            target_peers: config.target_peers,
            config,
            peers: HashMap::new(),
            banned_peers: HashMap::new(),
        })
    }
    
    /// Start peer manager
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Initialize peer discovery and connection logic
        Ok(())
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
}
