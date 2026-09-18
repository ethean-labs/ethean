//! Gossipsub service for message propagation
//!
//! Implements gossipsub protocol for efficient message broadcasting
//! across the beacon chain network.

use super::{NetworkError, NetworkMessage};
use crate::network::network_config::GossipConfig;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Gossip message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Message ID for deduplication
    pub message_id: String,
    /// Topic name
    pub topic: String,
    /// Message payload
    pub data: NetworkMessage,
    /// Timestamp when message was created
    pub timestamp: u64,
    /// Sender peer ID
    pub sender: Option<String>,
    /// Message signature (if enabled)
    pub signature: Option<Vec<u8>>,
}

impl GossipMessage {
    /// Create new gossip message
    pub fn new(topic: String, data: NetworkMessage) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Simple message ID generation (in production, use proper hash)
        let message_id = format!("{}_{}", topic, timestamp);
        
        Self {
            message_id,
            topic,
            data,
            timestamp,
            sender: None,
            signature: None,
        }
    }
    
    /// Check if message is expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        now - self.timestamp > max_age.as_secs()
    }
    
    /// Validate message structure
    pub fn validate(&self) -> bool {
        !self.message_id.is_empty() && 
        !self.topic.is_empty() &&
        self.timestamp > 0
    }
}

/// Topic subscription information
#[derive(Debug, Clone)]
pub struct TopicSubscription {
    /// Topic name
    pub topic: String,
    /// Subscribed peers
    pub peers: Vec<String>,
    /// Message count for this topic
    pub message_count: u64,
    /// Last activity timestamp
    pub last_activity: Instant,
}

impl TopicSubscription {
    /// Create new topic subscription
    pub fn new(topic: String) -> Self {
        Self {
            topic,
            peers: Vec::new(),
            message_count: 0,
            last_activity: Instant::now(),
        }
    }
    
    /// Add peer to subscription
    pub fn add_peer(&mut self, peer_id: String) {
        if !self.peers.contains(&peer_id) {
            self.peers.push(peer_id);
        }
        self.last_activity = Instant::now();
    }
    
    /// Remove peer from subscription
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p != peer_id);
        self.last_activity = Instant::now();
    }
    
    /// Increment message count
    pub fn increment_messages(&mut self) {
        self.message_count += 1;
        self.last_activity = Instant::now();
    }
}

/// Gossipsub service implementation
#[derive(Debug)]
pub struct GossipService {
    /// Configuration
    config: GossipConfig,
    /// Topic subscriptions
    subscriptions: HashMap<String, TopicSubscription>,
    /// Message cache for deduplication
    message_cache: HashMap<String, GossipMessage>,
    /// Pending outbound messages
    outbound_queue: VecDeque<GossipMessage>,
    /// Pending inbound messages
    inbound_queue: VecDeque<GossipMessage>,
    /// Performance metrics
    total_messages_sent: u64,
    total_messages_received: u64,
    total_bytes_sent: u64,
    total_bytes_received: u64,
    last_heartbeat: Instant,
}

impl GossipService {
    /// Create new gossip service
    pub fn new(config: GossipConfig) -> Result<Self, NetworkError> {
        Ok(Self {
            config,
            subscriptions: HashMap::new(),
            message_cache: HashMap::new(),
            outbound_queue: VecDeque::new(),
            inbound_queue: VecDeque::new(),
            total_messages_sent: 0,
            total_messages_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            last_heartbeat: Instant::now(),
        })
    }
    
    /// Start gossip service
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        // Initialize gossipsub protocol
        self.last_heartbeat = Instant::now();
        Ok(())
    }
    
    /// Subscribe to topic
    pub fn subscribe(&mut self, topic: &str) -> Result<(), NetworkError> {
        if !self.subscriptions.contains_key(topic) {
            let subscription = TopicSubscription::new(topic.to_string());
            self.subscriptions.insert(topic.to_string(), subscription);
        }
        Ok(())
    }
    
    /// Unsubscribe from topic
    pub fn unsubscribe(&mut self, topic: &str) -> Result<(), NetworkError> {
        self.subscriptions.remove(topic);
        Ok(())
    }
    
    /// Broadcast message to topic
    pub async fn broadcast(&mut self, topic: &str, message: &NetworkMessage) -> Result<(), NetworkError> {
        // Check if we're subscribed to this topic
        if !self.subscriptions.contains_key(topic) {
            return Err(NetworkError::GossipFailed { 
                topic: topic.to_string() 
            });
        }
        
        let gossip_msg = GossipMessage::new(topic.to_string(), message.clone());
        
        // Validate message size
        let message_size = self.estimate_message_size(&gossip_msg)?;
        if message_size > self.config.max_message_size {
            return Err(NetworkError::MessageValidation { 
                reason: format!("Message size {} exceeds limit {}", 
                    message_size, self.config.max_message_size)
            });
        }
        
        // Add to outbound queue
        self.outbound_queue.push_back(gossip_msg.clone());
        
        // Update metrics
        self.total_messages_sent += 1;
        self.total_bytes_sent += message_size as u64;
        
        // Update topic subscription
        if let Some(subscription) = self.subscriptions.get_mut(topic) {
            subscription.increment_messages();
        }
        
        // Cache message for deduplication
        self.message_cache.insert(gossip_msg.message_id.clone(), gossip_msg);
        
        Ok(())
    }
    
    /// Receive message from network
    pub fn receive_message(&mut self, message: GossipMessage) -> Result<bool, NetworkError> {
        // Validate message
        if !message.validate() {
            return Err(NetworkError::MessageValidation { 
                reason: "Invalid message structure".to_string() 
            });
        }
        
        // Check for duplicate
        if self.message_cache.contains_key(&message.message_id) {
            return Ok(false); // Duplicate message
        }
        
        // Check if message is expired
        if message.is_expired(Duration::from_secs(300)) { // 5 minutes
            return Ok(false); // Expired message
        }
        
        // Check if we're subscribed to this topic
        if !self.subscriptions.contains_key(&message.topic) {
            return Ok(false); // Not subscribed
        }
        
        let message_size = self.estimate_message_size(&message)?;
        
        // Add to inbound queue
        self.inbound_queue.push_back(message.clone());
        
        // Update metrics
        self.total_messages_received += 1;
        self.total_bytes_received += message_size as u64;
        
        // Cache message
        self.message_cache.insert(message.message_id.clone(), message);
        
        Ok(true) // Message accepted
    }
    
    /// Get next inbound message
    pub fn get_next_message(&mut self) -> Option<GossipMessage> {
        self.inbound_queue.pop_front()
    }
    
    /// Get next outbound message
    pub fn get_next_outbound(&mut self) -> Option<GossipMessage> {
        self.outbound_queue.pop_front()
    }
    
    /// Add peer to topic subscription
    pub fn add_peer_to_topic(&mut self, topic: &str, peer_id: String) -> Result<(), NetworkError> {
        if let Some(subscription) = self.subscriptions.get_mut(topic) {
            subscription.add_peer(peer_id);
            Ok(())
        } else {
            Err(NetworkError::GossipFailed { 
                topic: topic.to_string() 
            })
        }
    }
    
    /// Remove peer from topic subscription
    pub fn remove_peer_from_topic(&mut self, topic: &str, peer_id: &str) -> Result<(), NetworkError> {
        if let Some(subscription) = self.subscriptions.get_mut(topic) {
            subscription.remove_peer(peer_id);
            Ok(())
        } else {
            Err(NetworkError::GossipFailed { 
                topic: topic.to_string() 
            })
        }
    }
    
    /// Perform heartbeat maintenance
    pub fn heartbeat(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_heartbeat) >= self.config.heartbeat_interval {
            self.cleanup_cache();
            self.last_heartbeat = now;
        }
    }
    
    /// Cleanup expired messages from cache
    pub fn cleanup_cache(&mut self) {
        let max_age = Duration::from_secs(600); // 10 minutes
        self.message_cache.retain(|_, msg| !msg.is_expired(max_age));
        
        // Limit cache size
        if self.message_cache.len() > self.config.history_length * 100 {
            // Remove oldest messages (simple implementation)
            let oldest_keys: Vec<String> = self.message_cache.keys()
                .take(self.message_cache.len() - self.config.history_length * 100)
                .cloned()
                .collect();
                
            for key in oldest_keys {
                self.message_cache.remove(&key);
            }
        }
    }
    
    /// Estimate message size for validation
    fn estimate_message_size(&self, message: &GossipMessage) -> Result<usize, NetworkError> {
        // Simple size estimation (in production, use proper serialization)
        let base_size = message.topic.len() + message.message_id.len() + 8; // timestamp
        let data_size = match &message.data {
            NetworkMessage::BeaconBlock(_) => 512 * 1024, // Rough estimate
            NetworkMessage::Attestation(_) => 128,
            NetworkMessage::BlockRequest { .. } => 64,
            NetworkMessage::BlockResponse { blocks } => blocks.len() * 512 * 1024,
            NetworkMessage::StatusRequest => 32,
            NetworkMessage::StatusResponse { .. } => 64,
        };
        
        Ok(base_size + data_size)
    }
    
    /// Get gossip service statistics
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("subscriptions".to_string(), 
            serde_json::Value::Number(self.subscriptions.len().into()));
        stats.insert("cached_messages".to_string(), 
            serde_json::Value::Number(self.message_cache.len().into()));
        stats.insert("outbound_queue".to_string(), 
            serde_json::Value::Number(self.outbound_queue.len().into()));
        stats.insert("inbound_queue".to_string(), 
            serde_json::Value::Number(self.inbound_queue.len().into()));
        stats.insert("total_sent".to_string(), 
            serde_json::Value::Number(self.total_messages_sent.into()));
        stats.insert("total_received".to_string(), 
            serde_json::Value::Number(self.total_messages_received.into()));
        stats.insert("bytes_sent".to_string(), 
            serde_json::Value::Number(self.total_bytes_sent.into()));
        stats.insert("bytes_received".to_string(), 
            serde_json::Value::Number(self.total_bytes_received.into()));
            
        stats
    }
    
    /// Get subscribed topics
    pub fn get_subscribed_topics(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BeaconBlock, Slot};
    
    #[test]
    fn test_gossip_message_creation() {
        let data = NetworkMessage::StatusRequest;
        let msg = GossipMessage::new("test_topic".to_string(), data);
        
        assert_eq!(msg.topic, "test_topic");
        assert!(msg.validate());
        assert!(!msg.is_expired(Duration::from_secs(600)));
    }
    
    #[test]
    fn test_topic_subscription() {
        let mut subscription = TopicSubscription::new("test_topic".to_string());
        assert_eq!(subscription.peers.len(), 0);
        
        subscription.add_peer("peer1".to_string());
        assert_eq!(subscription.peers.len(), 1);
        
        subscription.add_peer("peer1".to_string()); // Duplicate
        assert_eq!(subscription.peers.len(), 1);
        
        subscription.remove_peer("peer1");
        assert_eq!(subscription.peers.len(), 0);
    }
    
    #[test]
    fn test_gossip_service_creation() {
        let config = GossipConfig::default();
        let service = GossipService::new(config);
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert_eq!(service.subscriptions.len(), 0);
        assert_eq!(service.total_messages_sent, 0);
    }
    
    #[test]
    fn test_topic_subscription_management() {
        let config = GossipConfig::default();
        let mut service = GossipService::new(config).unwrap();
        
        let result = service.subscribe("test_topic");
        assert!(result.is_ok());
        assert_eq!(service.subscriptions.len(), 1);
        
        let result = service.unsubscribe("test_topic");
        assert!(result.is_ok());
        assert_eq!(service.subscriptions.len(), 0);
    }
    
    #[tokio::test]
    async fn test_message_broadcasting() {
        let config = GossipConfig::default();
        let mut service = GossipService::new(config).unwrap();
        
        // Subscribe to topic first
        service.subscribe("test_topic").unwrap();
        
        let message = NetworkMessage::StatusRequest;
        let result = service.broadcast("test_topic", &message).await;
        assert!(result.is_ok());
        
        assert_eq!(service.total_messages_sent, 1);
        assert_eq!(service.outbound_queue.len(), 1);
    }
    
    #[test]
    fn test_message_reception() {
        let config = GossipConfig::default();
        let mut service = GossipService::new(config).unwrap();
        
        // Subscribe to topic
        service.subscribe("test_topic").unwrap();
        
        let data = NetworkMessage::StatusRequest;
        let message = GossipMessage::new("test_topic".to_string(), data);
        
        let result = service.receive_message(message);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        
        assert_eq!(service.total_messages_received, 1);
        assert_eq!(service.inbound_queue.len(), 1);
    }
    
    #[test]
    fn test_duplicate_message_detection() {
        let config = GossipConfig::default();
        let mut service = GossipService::new(config).unwrap();
        
        service.subscribe("test_topic").unwrap();
        
        let data = NetworkMessage::StatusRequest;
        let message = GossipMessage::new("test_topic".to_string(), data);
        
        // Send same message twice
        let result1 = service.receive_message(message.clone());
        let result2 = service.receive_message(message);
        
        assert!(result1.is_ok() && result1.unwrap());
        assert!(result2.is_ok() && !result2.unwrap()); // Duplicate
        
        assert_eq!(service.total_messages_received, 1);
    }
}
