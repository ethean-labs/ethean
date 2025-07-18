//! Peer discovery service using Discovery v5
//!
//! Implements peer discovery protocol for finding and connecting
//! to other Beam Chain nodes on the network.

use super::NetworkError;
use crate::network::network_config::DiscoveryConfig;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Node information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNode {
    /// Node ID (public key hash)
    pub node_id: String,
    /// IP address
    pub ip: IpAddr,
    /// TCP port for connections
    pub tcp_port: u16,
    /// UDP port for discovery
    pub udp_port: u16,
    /// Node capabilities/protocols
    pub capabilities: Vec<String>,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Discovery score (reputation)
    pub score: i32,
    /// Connection attempts
    pub connection_attempts: u32,
}

impl DiscoveryNode {
    /// Create new discovery node
    pub fn new(node_id: String, addr: SocketAddr) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            node_id,
            ip: addr.ip(),
            tcp_port: addr.port(),
            udp_port: addr.port(),
            capabilities: vec!["beacon".to_string()],
            last_seen: timestamp,
            score: 50, // Neutral score
            connection_attempts: 0,
        }
    }
    
    /// Get socket address for TCP connections
    pub fn tcp_address(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.tcp_port)
    }
    
    /// Get socket address for UDP discovery
    pub fn udp_address(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.udp_port)
    }
    
    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Check if node has capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
    
    /// Add capability
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }
    
    /// Apply score penalty
    pub fn apply_penalty(&mut self, penalty: i32) {
        self.score -= penalty;
        if self.score < -100 {
            self.score = -100;
        }
    }
    
    /// Apply score reward
    pub fn apply_reward(&mut self, reward: i32) {
        self.score += reward;
        if self.score > 100 {
            self.score = 100;
        }
    }
    
    /// Check if node should be banned
    pub fn should_ban(&self) -> bool {
        self.score < -50 || self.connection_attempts > 10
    }
}

/// Discovery query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryQuery {
    /// Find nodes near target ID
    FindNode { target: String, count: usize },
    /// Ping node for liveness
    Ping { timestamp: u64 },
    /// Pong response to ping
    Pong { timestamp: u64 },
    /// Node capabilities announcement
    Capabilities { protocols: Vec<String> },
}

/// Discovery message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    /// Message ID for tracking
    pub id: String,
    /// Source node ID
    pub from: String,
    /// Target node ID (empty for broadcast)
    pub to: String,
    /// Query type
    pub query: DiscoveryQuery,
    /// Message timestamp
    pub timestamp: u64,
}

impl DiscoveryMessage {
    /// Create new discovery message
    pub fn new(from: String, to: String, query: DiscoveryQuery) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let id = format!("{}_{}", from, timestamp);
        
        Self {
            id,
            from,
            to,
            query,
            timestamp,
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
}

/// Discovery service implementation
#[derive(Debug)]
pub struct DiscoveryService {
    /// Configuration
    config: DiscoveryConfig,
    /// Our node ID
    local_node_id: String,
    /// Discovered nodes table
    node_table: HashMap<String, DiscoveryNode>,
    /// Bootstrap nodes
    bootstrap_nodes: Vec<DiscoveryNode>,
    /// Pending queries
    pending_queries: HashMap<String, DiscoveryMessage>,
    /// Active connections set
    active_connections: HashSet<String>,
    /// Discovery statistics
    total_queries_sent: u64,
    total_responses_received: u64,
    last_discovery_round: Instant,
    discovery_enabled: bool,
}

impl DiscoveryService {
    /// Create new discovery service
    pub fn new(config: DiscoveryConfig) -> Result<Self, NetworkError> {
        // Generate random node ID (in production, derive from key)
        let local_node_id = format!("panro_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());
        
        let mut bootstrap_nodes = Vec::new();
        
        // Parse bootstrap node addresses
        for bootstrap in &config.bootstrap_nodes {
            if let Some(node) = Self::parse_bootstrap_node(bootstrap) {
                bootstrap_nodes.push(node);
            }
        }
        
        Ok(Self {
            discovery_enabled: config.enabled,
            config,
            local_node_id,
            node_table: HashMap::new(),
            bootstrap_nodes,
            pending_queries: HashMap::new(),
            active_connections: HashSet::new(),
            total_queries_sent: 0,
            total_responses_received: 0,
            last_discovery_round: Instant::now(),
        })
    }
    
    /// Start discovery service
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        if !self.discovery_enabled {
            return Ok(());
        }
        
        // Add bootstrap nodes to table
        for node in &self.bootstrap_nodes {
            self.node_table.insert(node.node_id.clone(), node.clone());
        }
        
        // Start initial discovery round
        self.run_discovery_round().await?;
        
        Ok(())
    }
    
    /// Run discovery round to find new peers
    pub async fn run_discovery_round(&mut self) -> Result<(), NetworkError> {
        if !self.discovery_enabled {
            return Ok(());
        }
        
        let now = Instant::now();
        if now.duration_since(self.last_discovery_round) < self.config.discovery_interval {
            return Ok(()); // Too soon for next round
        }
        
        // Send find_node queries to known nodes
        let target_nodes: Vec<String> = self.node_table.keys()
            .take(self.config.max_nodes_per_discovery)
            .cloned()
            .collect();
            
        for node_id in target_nodes {
            self.send_find_node_query(&node_id).await?;
        }
        
        self.last_discovery_round = now;
        Ok(())
    }
    
    /// Send find_node query to discover peers
    async fn send_find_node_query(&mut self, target_node: &str) -> Result<(), NetworkError> {
        let query = DiscoveryQuery::FindNode {
            target: self.local_node_id.clone(),
            count: 20,
        };
        
        let message = DiscoveryMessage::new(
            self.local_node_id.clone(),
            target_node.to_string(),
            query,
        );
        
        self.pending_queries.insert(message.id.clone(), message.clone());
        self.total_queries_sent += 1;
        
        // In production, send actual network message
        Ok(())
    }
    
    /// Handle incoming discovery message
    pub fn handle_message(&mut self, message: DiscoveryMessage) -> Result<Option<DiscoveryMessage>, NetworkError> {
        if message.is_expired(Duration::from_secs(30)) {
            return Ok(None); // Ignore expired messages
        }
        
        match message.query {
            DiscoveryQuery::FindNode { target, count } => {
                self.handle_find_node(&message.from, &target, count)
            },
            DiscoveryQuery::Ping { timestamp } => {
                self.handle_ping(&message.from, timestamp)
            },
            DiscoveryQuery::Pong { timestamp } => {
                self.handle_pong(&message.from, timestamp);
                Ok(None)
            },
            DiscoveryQuery::Capabilities { protocols } => {
                self.handle_capabilities(&message.from, protocols);
                Ok(None)
            },
        }
    }
    
    /// Handle find_node query
    fn handle_find_node(&mut self, from: &str, _target: &str, count: usize) -> Result<Option<DiscoveryMessage>, NetworkError> {
        // Find closest nodes to target
        let _closest_nodes: Vec<&DiscoveryNode> = self.node_table.values()
            .filter(|node| node.node_id != *from) // Don't return sender
            .take(count)
            .collect();
        
        // In production, return actual node list
        // For now, just acknowledge the query
        let response = DiscoveryMessage::new(
            self.local_node_id.clone(),
            from.to_string(),
            DiscoveryQuery::Pong { 
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            },
        );
        
        Ok(Some(response))
    }
    
    /// Handle ping query
    fn handle_ping(&mut self, from: &str, timestamp: u64) -> Result<Option<DiscoveryMessage>, NetworkError> {
        // Update node in table if exists
        if let Some(node) = self.node_table.get_mut(from) {
            node.update_last_seen();
        }
        
        // Respond with pong
        let response = DiscoveryMessage::new(
            self.local_node_id.clone(),
            from.to_string(),
            DiscoveryQuery::Pong { timestamp },
        );
        
        Ok(Some(response))
    }
    
    /// Handle pong response
    fn handle_pong(&mut self, from: &str, _timestamp: u64) {
        if let Some(node) = self.node_table.get_mut(from) {
            node.update_last_seen();
            node.apply_reward(1); // Good behavior
        }
        
        self.total_responses_received += 1;
    }
    
    /// Handle capabilities announcement
    fn handle_capabilities(&mut self, from: &str, protocols: Vec<String>) {
        if let Some(node) = self.node_table.get_mut(from) {
            node.capabilities = protocols;
            node.update_last_seen();
        }
    }
    
    /// Add discovered node to table
    pub fn add_discovered_node(&mut self, node: DiscoveryNode) -> Result<(), NetworkError> {
        if self.node_table.len() >= self.config.table_size_limit {
            // Remove oldest node
            if let Some((oldest_id, _)) = self.node_table.iter()
                .min_by_key(|(_, node)| node.last_seen)
                .map(|(id, node)| (id.clone(), node.clone()))
            {
                self.node_table.remove(&oldest_id);
            }
        }
        
        self.node_table.insert(node.node_id.clone(), node);
        Ok(())
    }
    
    /// Get nodes suitable for connection
    pub fn get_connectable_nodes(&self, max_count: usize) -> Vec<&DiscoveryNode> {
        self.node_table.values()
            .filter(|node| {
                !self.active_connections.contains(&node.node_id) &&
                !node.should_ban() &&
                node.has_capability("beacon")
            })
            .take(max_count)
            .collect()
    }
    
    /// Mark node as connected
    pub fn mark_connected(&mut self, node_id: &str) {
        self.active_connections.insert(node_id.to_string());
        if let Some(node) = self.node_table.get_mut(node_id) {
            node.apply_reward(10); // Connection success
        }
    }
    
    /// Mark node as disconnected
    pub fn mark_disconnected(&mut self, node_id: &str) {
        self.active_connections.remove(node_id);
    }
    
    /// Cleanup expired nodes and queries
    pub fn cleanup(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Remove nodes not seen for 1 hour
        let stale_threshold = now - 3600;
        self.node_table.retain(|_, node| node.last_seen > stale_threshold);
        
        // Remove expired queries
        self.pending_queries.retain(|_, msg| !msg.is_expired(Duration::from_secs(60)));
    }
    
    /// Parse bootstrap node from string
    fn parse_bootstrap_node(bootstrap: &str) -> Option<DiscoveryNode> {
        // Simple parser for /ip4/address/tcp/port format
        if bootstrap.starts_with("/ip4/") {
            let parts: Vec<&str> = bootstrap.split('/').collect();
            if parts.len() >= 5 {
                if let (Ok(ip), Ok(port)) = (parts[2].parse::<IpAddr>(), parts[4].parse::<u16>()) {
                    let addr = SocketAddr::new(ip, port);
                    let node_id = format!("bootstrap_{}", parts[2]);
                    return Some(DiscoveryNode::new(node_id, addr));
                }
            }
        }
        None
    }
    
    /// Get discovery statistics
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("enabled".to_string(), 
            serde_json::Value::Bool(self.discovery_enabled));
        stats.insert("node_table_size".to_string(), 
            serde_json::Value::Number(self.node_table.len().into()));
        stats.insert("active_connections".to_string(), 
            serde_json::Value::Number(self.active_connections.len().into()));
        stats.insert("pending_queries".to_string(), 
            serde_json::Value::Number(self.pending_queries.len().into()));
        stats.insert("queries_sent".to_string(), 
            serde_json::Value::Number(self.total_queries_sent.into()));
        stats.insert("responses_received".to_string(), 
            serde_json::Value::Number(self.total_responses_received.into()));
            
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_discovery_node_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);
        let node = DiscoveryNode::new("test_node".to_string(), addr);
        
        assert_eq!(node.node_id, "test_node");
        assert_eq!(node.tcp_port, 9000);
        assert!(node.has_capability("beacon"));
    }
    
    #[test]
    fn test_discovery_node_scoring() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);
        let mut node = DiscoveryNode::new("test_node".to_string(), addr);
        
        assert_eq!(node.score, 50);
        
        node.apply_reward(20);
        assert_eq!(node.score, 70);
        
        node.apply_penalty(30);
        assert_eq!(node.score, 40);
    }
    
    #[test]
    fn test_discovery_message_creation() {
        let query = DiscoveryQuery::Ping { timestamp: 12345 };
        let message = DiscoveryMessage::new("from".to_string(), "to".to_string(), query);
        
        assert_eq!(message.from, "from");
        assert_eq!(message.to, "to");
        assert!(!message.is_expired(Duration::from_secs(60)));
    }
    
    #[test]
    fn test_discovery_service_creation() {
        let config = DiscoveryConfig::default();
        let service = DiscoveryService::new(config);
        
        assert!(service.is_ok());
        let service = service.unwrap();
        assert!(!service.local_node_id.is_empty());
        assert_eq!(service.node_table.len(), 0);
    }
    
    #[test]
    fn test_bootstrap_node_parsing() {
        let bootstrap = "/ip4/127.0.0.1/tcp/9000";
        let node = DiscoveryService::parse_bootstrap_node(bootstrap);
        
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.tcp_port, 9000);
        assert_eq!(node.ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }
    
    #[test]
    fn test_node_table_management() {
        let config = DiscoveryConfig::default();
        let mut service = DiscoveryService::new(config).unwrap();
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9000);
        let node = DiscoveryNode::new("test_node".to_string(), addr);
        
        let result = service.add_discovered_node(node);
        assert!(result.is_ok());
        assert_eq!(service.node_table.len(), 1);
        
        service.mark_connected("test_node");
        assert!(service.active_connections.contains("test_node"));
        
        service.mark_disconnected("test_node");
        assert!(!service.active_connections.contains("test_node"));
    }
}
