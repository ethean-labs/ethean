//! Network configuration for P2P layer
//!
//! Defines configuration structures for peer management, gossip,
//! discovery, and overall network behavior.

use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Peer manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    /// Maximum number of connected peers
    pub max_peers: u32,
    /// Target number of peers to maintain
    pub target_peers: u32,
    /// Minimum number of peers before seeking more
    pub min_peers: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Peer scoring enabled
    pub enable_scoring: bool,
    /// Peer ban duration for bad behavior
    pub ban_duration_secs: u64,
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            target_peers: 30,
            min_peers: 10,
            connection_timeout: Duration::from_secs(10),
            enable_scoring: true,
            ban_duration_secs: 3600, // 1 hour
        }
    }
}

/// Gossipsub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Gossipsub mesh degree (D)
    pub mesh_n: usize,
    /// Gossipsub mesh degree low watermark (D_lo)
    pub mesh_n_low: usize,
    /// Gossipsub mesh degree high watermark (D_hi) 
    pub mesh_n_high: usize,
    /// Gossip factor for message propagation
    pub gossip_factor: f64,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Message cache duration
    pub history_length: usize,
    /// Maximum message size
    pub max_message_size: usize,
    /// Enable message signing
    pub enable_signing: bool,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            mesh_n: 6,
            mesh_n_low: 4,
            mesh_n_high: 12,
            gossip_factor: 0.25,
            heartbeat_interval: Duration::from_millis(700),
            history_length: 5,
            max_message_size: 1024 * 1024, // 1MB
            enable_signing: true,
        }
    }
}

/// Discovery service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable discovery service
    pub enabled: bool,
    /// Bootstrap nodes for initial discovery
    pub bootstrap_nodes: Vec<String>,
    /// Discovery port
    pub port: u16,
    /// Discovery interval
    pub discovery_interval: Duration,
    /// Maximum nodes to discover per round
    pub max_nodes_per_discovery: usize,
    /// Node table size limit
    pub table_size_limit: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bootstrap_nodes: vec![
                // Ethereum mainnet bootstrap nodes
                "/ip4/18.138.108.67/tcp/9000/p2p/16Uiu2HAm7Qwe19vz9WzD2Mxn7fXd1vgHHp4iccuyq7TxwRXoAGfc".to_string(),
                "/ip4/52.59.65.77/tcp/9000/p2p/16Uiu2HAm2ZjWqw5PUu8gRMEojEHCNUjpYYwRbRVa1b8G6yPHFUh6".to_string(),
            ],
            port: 9000,
            discovery_interval: Duration::from_secs(30),
            max_nodes_per_discovery: 20,
            table_size_limit: 1000,
        }
    }
}

/// Message handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfig {
    /// Maximum pending messages
    pub max_pending_messages: usize,
    /// Message processing timeout
    pub message_timeout: Duration,
    /// Enable message validation
    pub enable_validation: bool,
    /// Maximum message retry attempts
    pub max_retries: u32,
    /// Message priority levels
    pub enable_priority: bool,
}

impl Default for MessageConfig {
    fn default() -> Self {
        Self {
            max_pending_messages: 1000,
            message_timeout: Duration::from_secs(5),
            enable_validation: true,
            max_retries: 3,
            enable_priority: true,
        }
    }
}

/// Main network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address for incoming connections
    pub listen_address: SocketAddr,
    /// External address to advertise
    pub external_address: Option<SocketAddr>,
    /// Enable IPv6 support
    pub enable_ipv6: bool,
    /// Network name/chain ID
    pub network_name: String,
    /// Protocol version
    pub protocol_version: String,
    /// Client version string
    pub client_version: String,
    /// Peer management configuration
    pub peer_config: PeerConfig,
    /// Gossipsub configuration
    pub gossip_config: GossipConfig,
    /// Discovery configuration
    pub discovery_config: DiscoveryConfig,
    /// Message handling configuration
    pub message_config: MessageConfig,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics server address
    pub metrics_address: Option<SocketAddr>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:9000".parse().unwrap(),
            external_address: None,
            enable_ipv6: false,
            network_name: "beacon-chain".to_string(),
            protocol_version: "/beam/1.0.0".to_string(),
            client_version: "ethean/0.1.0".to_string(),
            peer_config: PeerConfig::default(),
            gossip_config: GossipConfig::default(),
            discovery_config: DiscoveryConfig::default(),
            message_config: MessageConfig::default(),
            enable_metrics: true,
            metrics_address: Some("127.0.0.1:9090".parse().unwrap()),
        }
    }
}

impl NetworkConfig {
    /// Create configuration for testnet
    pub fn testnet() -> Self {
        let mut config = Self::default();
        config.network_name = "beacon-chain-testnet".to_string();
        config.listen_address = "0.0.0.0:9001".parse().unwrap();
        config.discovery_config.port = 9001;
        config
    }
    
    /// Create configuration for local development
    pub fn local() -> Self {
        let mut config = Self::default();
        config.network_name = "beacon-chain-local".to_string();
        config.listen_address = "127.0.0.1:9000".parse().unwrap();
        config.discovery_config.enabled = false;
        config.discovery_config.bootstrap_nodes.clear();
        config.peer_config.max_peers = 10;
        config.peer_config.target_peers = 5;
        config
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.peer_config.min_peers > self.peer_config.target_peers {
            return Err("min_peers cannot be greater than target_peers".to_string());
        }
        
        if self.peer_config.target_peers > self.peer_config.max_peers {
            return Err("target_peers cannot be greater than max_peers".to_string());
        }
        
        if self.gossip_config.mesh_n_low > self.gossip_config.mesh_n {
            return Err("mesh_n_low cannot be greater than mesh_n".to_string());
        }
        
        if self.gossip_config.mesh_n > self.gossip_config.mesh_n_high {
            return Err("mesh_n cannot be greater than mesh_n_high".to_string());
        }
        
        if self.gossip_config.max_message_size > 10 * 1024 * 1024 {
            return Err("max_message_size cannot exceed 10MB".to_string());
        }
        
        Ok(())
    }
    
    /// Get gossipsub topics for this network
    pub fn get_gossip_topics(&self) -> Vec<String> {
        vec![
            format!("{}/beacon_block", self.network_name),
            format!("{}/beacon_attestation", self.network_name),
            format!("{}/voluntary_exit", self.network_name),
            format!("{}/proposer_slashing", self.network_name),
            format!("{}/attester_slashing", self.network_name),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peer_config_default() {
        let config = PeerConfig::default();
        assert_eq!(config.max_peers, 50);
        assert_eq!(config.target_peers, 30);
        assert_eq!(config.min_peers, 10);
        assert!(config.enable_scoring);
    }
    
    #[test]
    fn test_gossip_config_default() {
        let config = GossipConfig::default();
        assert_eq!(config.mesh_n, 6);
        assert_eq!(config.mesh_n_low, 4);
        assert_eq!(config.mesh_n_high, 12);
        assert!(config.enable_signing);
    }
    
    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 9000);
        assert!(!config.bootstrap_nodes.is_empty());
    }
    
    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.network_name, "beacon-chain");
        assert_eq!(config.protocol_version, "/beam/1.0.0");
        assert_eq!(config.client_version, "ethean/0.1.0");
        assert!(config.enable_metrics);
    }
    
    #[test]
    fn test_network_config_testnet() {
        let config = NetworkConfig::testnet();
        assert_eq!(config.network_name, "beacon-chain-testnet");
        assert_eq!(config.discovery_config.port, 9001);
    }
    
    #[test]
    fn test_network_config_local() {
        let config = NetworkConfig::local();
        assert_eq!(config.network_name, "beacon-chain-local");
        assert!(!config.discovery_config.enabled);
        assert_eq!(config.peer_config.max_peers, 10);
    }
    
    #[test]
    fn test_config_validation_valid() {
        let config = NetworkConfig::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation_invalid_peer_counts() {
        let mut config = NetworkConfig::default();
        config.peer_config.min_peers = 40;
        config.peer_config.target_peers = 30;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("min_peers cannot be greater"));
    }
    
    #[test]
    fn test_config_validation_invalid_mesh() {
        let mut config = NetworkConfig::default();
        config.gossip_config.mesh_n_low = 10;
        config.gossip_config.mesh_n = 5;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mesh_n_low cannot be greater"));
    }
    
    #[test]
    fn test_gossip_topics() {
        let config = NetworkConfig::default();
        let topics = config.get_gossip_topics();
        assert_eq!(topics.len(), 5);
        assert!(topics[0].contains("beacon_block"));
        assert!(topics[1].contains("beacon_attestation"));
    }
}
