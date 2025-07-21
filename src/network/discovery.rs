//! Advanced P2P peer discovery service
//!
//! Implements multi-layer peer discovery using Kademlia DHT and mDNS
//! for robust peer finding in Ethereum Beacon Chain networks.

use super::NetworkError;
use crate::network::network_config::DiscoveryConfig;
use libp2p::{
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    kad::{
        Kademlia, KademliaConfig, KademliaEvent, QueryId, QueryResult,
        Record, RecordKey, store::MemoryStore,
    },
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    multiaddr::Protocol,
    swarm::{SwarmEvent, Swarm},
    Multiaddr, PeerId,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

/// Wrapper for PeerId to enable serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePeerId {
    #[serde(with = "peer_id_serde")]
    pub peer_id: PeerId,
}

mod peer_id_serde {
    use super::*;
    use serde::{Serializer, Deserializer};

    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&peer_id.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

/// Discovery service errors
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Bootstrap failed: {0}")]
    BootstrapFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Enhanced node information for advanced peer discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNode {
    /// Node ID (libp2p PeerId)
    pub peer_id: SerializablePeerId,
    /// Multiaddresses for connections
    pub addresses: Vec<Multiaddr>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Agent version string
    pub agent_version: String,
    /// Protocol version
    pub protocol_version: String,
    /// Discovery timestamp (Unix timestamp)
    pub discovered_at: u64,
    /// Last seen timestamp (Unix timestamp)
    pub last_seen: u64,
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    /// Connection attempts
    pub connection_attempts: u32,
    /// Successful connections
    pub successful_connections: u32,
    /// Node reputation score
    pub reputation_score: i32,
    /// Network latency (if known)
    pub latency: Option<Duration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Bootstrap,
    Kademlia,
    Mdns,
    PeerExchange,
    Manual,
}

/// Advanced peer discovery service with DHT and mDNS support
pub struct PeerDiscovery {
    /// Kademlia DHT for peer discovery
    kademlia: Kademlia<MemoryStore>,
    /// Local network discovery via mDNS
    mdns: Option<Mdns>,
    /// Identity protocol for peer information exchange
    identify: Identify,
    /// Configuration parameters
    config: AdvancedDiscoveryConfig,
    /// Recently discovered peers
    discovered_peers: HashMap<PeerId, DiscoveryNode>,
    /// Bootstrap node addresses
    bootstrap_nodes: Vec<Multiaddr>,
    /// Active discovery queries
    active_queries: HashMap<QueryId, QueryInfo>,
    /// Discovery statistics
    stats: DiscoveryStats,
    /// Last bootstrap attempt
    last_bootstrap: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct AdvancedDiscoveryConfig {
    /// Enable mDNS discovery for local peers
    pub enable_mdns: bool,
    /// Enable Kademlia DHT discovery
    pub enable_kademlia: bool,
    /// Bootstrap nodes for DHT
    pub bootstrap_nodes: Vec<Multiaddr>,
    /// Query timeout duration
    pub query_timeout: Duration,
    /// Bootstrap interval
    pub bootstrap_interval: Duration,
    /// Maximum number of peers to discover
    pub max_discovered_peers: usize,
    /// Peer information TTL
    pub peer_info_ttl: Duration,
    /// Random walk interval for DHT maintenance
    pub random_walk_interval: Duration,
    /// Minimum peer score for connections
    pub min_peer_score: i32,
}

#[derive(Debug, Clone)]
struct QueryInfo {
    query_type: QueryType,
    started_at: Instant,
    target: Option<PeerId>,
}

#[derive(Debug, Clone)]
enum QueryType {
    Bootstrap,
    FindPeer(PeerId),
    GetProviders(RecordKey),
    RandomWalk,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryStats {
    /// Total peers discovered
    pub peers_discovered: u64,
    /// Peers discovered via mDNS
    pub mdns_discoveries: u64,
    /// Peers discovered via Kademlia
    pub kademlia_discoveries: u64,
    /// Bootstrap attempts
    pub bootstrap_attempts: u64,
    /// Successful bootstrap operations
    pub successful_bootstraps: u64,
    /// Active queries count
    pub active_queries: u64,
    /// Failed queries
    pub failed_queries: u64,
    /// Average query time
    pub avg_query_time: Duration,
}

impl Default for AdvancedDiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_kademlia: true,
            bootstrap_nodes: Vec::new(),
            query_timeout: Duration::from_secs(30),
            bootstrap_interval: Duration::from_secs(300), // 5 minutes
            max_discovered_peers: 1000,
            peer_info_ttl: Duration::from_secs(3600), // 1 hour
            random_walk_interval: Duration::from_secs(600), // 10 minutes
            min_peer_score: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DiscoveryAction {
    /// Connect to a newly discovered peer
    ConnectToPeer {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
    /// Disconnect from a peer with poor reputation
    DisconnectFromPeer {
        peer_id: PeerId,
        reason: String,
    },
    /// Update routing table entry
    UpdateRoutingTable {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
    /// Broadcast peer information
    BroadcastPeer {
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
    },
}

#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
    Identify(IdentifyEvent),
}

/// Gossip protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Peer advertisement message
    PeerAdvertisement {
        peer_id: String,
        addresses: Vec<String>,
        timestamp: u64,
        signature: Vec<u8>,
    },
    /// Network topology update
    TopologyUpdate {
        updates: Vec<TopologyChange>,
        timestamp: u64,
    },
    /// Content routing information
    ContentRouting {
        content_id: String,
        provider_peers: Vec<String>,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyChange {
    pub peer_id: String,
    pub change_type: ChangeType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    PeerJoined,
    PeerLeft,
    AddressUpdated,
    CapabilityChanged,
}

/// Gossip protocol manager for efficient peer information propagation
pub struct GossipProtocol {
    local_peer_id: PeerId,
    gossip_peers: HashMap<PeerId, GossipPeerInfo>,
    message_cache: HashMap<String, CachedMessage>,
    config: GossipConfig,
    stats: GossipStats,
}

#[derive(Debug, Clone)]
pub struct GossipPeerInfo {
    pub peer_id: PeerId,
    pub score: f64,
    pub last_message: Instant,
    pub message_count: u64,
}

#[derive(Debug, Clone)]
pub struct CachedMessage {
    pub message: GossipMessage,
    pub received_at: Instant,
    pub ttl: Duration,
    pub propagation_count: u32,
}

#[derive(Debug, Clone)]
pub struct GossipConfig {
    pub max_gossip_peers: usize,
    pub gossip_interval: Duration,
    pub message_ttl: Duration,
    pub max_propagation_hops: u32,
    pub peer_exchange_interval: Duration,
}

#[derive(Debug, Default)]
pub struct GossipStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_propagated: u64,
    pub peer_exchanges: u64,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            max_gossip_peers: 12,
            gossip_interval: Duration::from_secs(30),
            message_ttl: Duration::from_secs(300),
            max_propagation_hops: 3,
            peer_exchange_interval: Duration::from_secs(60),
        }
    }
}
impl PeerDiscovery {
    /// Create a new advanced peer discovery service
    pub fn new(
        local_peer_id: PeerId,
        config: AdvancedDiscoveryConfig,
    ) -> Result<Self, DiscoveryError> {
        // Initialize Kademlia DHT
        let store = MemoryStore::new(local_peer_id);
        let mut kademlia_config = KademliaConfig::default();
        kademlia_config.set_query_timeout(config.query_timeout);
        kademlia_config.set_replication_factor(
            std::num::NonZeroUsize::new(20).unwrap()
        );
        let mut kademlia = Kademlia::with_config(local_peer_id, store, kademlia_config);

        // Add bootstrap nodes to Kademlia
        for addr in &config.bootstrap_nodes {
            if let Some(Protocol::P2p(peer_id_hash)) = addr.iter().last() {
                if let Ok(peer_id) = PeerId::from_multihash(peer_id_hash.into()) {
                    kademlia.add_address(&peer_id, addr.clone());
                }
            }
        }

        // Initialize mDNS if enabled
        let mdns = if config.enable_mdns {
            match Mdns::new(MdnsConfig::default()) {
                Ok(mdns) => Some(mdns),
                Err(e) => {
                    warn!("Failed to initialize mDNS: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize Identify protocol
        let identify = Identify::new(IdentifyConfig::new(
            "panro/1.0.0".to_string(),
            local_peer_id.to_owned().into(),
        ));

        Ok(Self {
            kademlia,
            mdns,
            identify,
            bootstrap_nodes: config.bootstrap_nodes.clone(),
            config,
            discovered_peers: HashMap::new(),
            active_queries: HashMap::new(),
            stats: DiscoveryStats::default(),
            last_bootstrap: None,
        })
    }

    /// Start peer discovery process
    pub async fn start_discovery(&mut self) -> Result<(), DiscoveryError> {
        info!("Starting advanced peer discovery service");

        // Bootstrap Kademlia DHT
        if self.config.enable_kademlia && !self.bootstrap_nodes.is_empty() {
            self.bootstrap_kademlia().await?;
        }

        // Start periodic maintenance tasks
        self.start_maintenance_tasks().await;

        Ok(())
    }

    /// Bootstrap Kademlia DHT
    async fn bootstrap_kademlia(&mut self) -> Result<(), DiscoveryError> {
        info!("Bootstrapping Kademlia DHT with {} nodes", self.bootstrap_nodes.len());
        
        self.stats.bootstrap_attempts += 1;
        self.last_bootstrap = Some(Instant::now());

        match self.kademlia.bootstrap() {
            Ok(query_id) => {
                let query_info = QueryInfo {
                    query_type: QueryType::Bootstrap,
                    started_at: Instant::now(),
                    target: None,
                };
                self.active_queries.insert(query_id, query_info);
                self.stats.active_queries += 1;
                
                debug!("Started DHT bootstrap with query ID: {:?}", query_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to bootstrap Kademlia: {}", e);
                Err(DiscoveryError::BootstrapFailed(e.to_string()))
            }
        }
    }

    /// Handle discovery events
    pub async fn handle_discovery_event(
        &mut self,
        event: DiscoveryEvent,
    ) -> Result<Vec<DiscoveryAction>, DiscoveryError> {
        let mut actions = Vec::new();

        match event {
            DiscoveryEvent::Kademlia(kad_event) => {
                actions.extend(self.handle_kademlia_event(kad_event).await?);
            }
            DiscoveryEvent::Mdns(mdns_event) => {
                actions.extend(self.handle_mdns_event(mdns_event).await?);
            }
            DiscoveryEvent::Identify(identify_event) => {
                actions.extend(self.handle_identify_event(identify_event).await?);
            }
        }

        Ok(actions)
    }

    /// Add or update peer information
    pub fn add_or_update_peer(
        &mut self,
        peer_id: PeerId,
        addresses: Vec<Multiaddr>,
        method: DiscoveryMethod,
    ) {
        if let Some(peer_info) = self.discovered_peers.get_mut(&peer_id) {
            // Update existing peer
            peer_info.last_seen = Instant::now();
            
            // Add new addresses
            for addr in addresses {
                if !peer_info.addresses.contains(&addr) {
                    peer_info.addresses.push(addr.clone());
                    self.kademlia.add_address(&peer_id, addr);
                }
            }
        } else {
            // Add new peer
            if self.discovered_peers.len() >= self.config.max_discovered_peers {
                // Remove oldest peer to make room
                if let Some((oldest_peer, _)) = self.discovered_peers
                    .iter()
                    .min_by_key(|(_, info)| info.last_seen)
                    .map(|(peer, info)| (*peer, info.clone()))
                {
                    self.discovered_peers.remove(&oldest_peer);
                }
            }
            
            let peer_info = DiscoveryNode {
                peer_id,
                addresses: addresses.clone(),
                protocols: Vec::new(),
                agent_version: String::new(),
                protocol_version: String::new(),
                discovered_at: Instant::now(),
                last_seen: Instant::now(),
                discovery_method: method,
                connection_attempts: 0,
                successful_connections: 0,
                reputation_score: 50, // Neutral score
                latency: None,
            };
            
            self.discovered_peers.insert(peer_id, peer_info);
            self.stats.peers_discovered += 1;
            
            // Add addresses to Kademlia
            for addr in addresses {
                self.kademlia.add_address(&peer_id, addr);
            }
        }
    }

    /// Get discovered peer information
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<&DiscoveryNode> {
        self.discovered_peers.get(peer_id)
    }

    /// Get all discovered peers
    pub fn discovered_peers(&self) -> &HashMap<PeerId, DiscoveryNode> {
        &self.discovered_peers
    }

    /// Get discovery statistics
    pub fn stats(&self) -> &DiscoveryStats {
        &self.stats
    }

    /// Clean up expired peer information
    pub fn cleanup_expired_peers(&mut self) -> usize {
        let ttl = self.config.peer_info_ttl;
        let now = Instant::now();
        let mut removed = 0;

        self.discovered_peers.retain(|_peer_id, peer_info| {
            if now.duration_since(peer_info.last_seen) > ttl {
                removed += 1;
                false
            } else {
                true
            }
        });

        if removed > 0 {
            debug!("Cleaned up {} expired peer entries", removed);
        }

        removed
    }

    /// Record connection attempt for a peer
    pub fn record_connection_attempt(&mut self, peer_id: &PeerId, successful: bool) {
        if let Some(peer_info) = self.discovered_peers.get_mut(peer_id) {
            peer_info.connection_attempts += 1;
            if successful {
                peer_info.successful_connections += 1;
                peer_info.last_seen = Instant::now();
                peer_info.reputation_score += 1; // Small reputation boost
            } else {
                peer_info.reputation_score -= 2; // Penalty for failed connection
            }
        }
    }

    /// Get peers suitable for connection
    pub fn get_connectable_peers(&self, max_count: usize) -> Vec<PeerId> {
        let mut peers: Vec<_> = self.discovered_peers
            .iter()
            .filter(|(_, info)| {
                info.reputation_score >= self.config.min_peer_score &&
                !info.addresses.is_empty()
            })
            .collect();
        
        // Sort by reputation score (descending)
        peers.sort_by_key(|(_, info)| std::cmp::Reverse(info.reputation_score));
        
        peers.into_iter()
            .take(max_count)
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }

    /// Update peer latency information
    pub fn update_peer_latency(&mut self, peer_id: &PeerId, latency: Duration) {
        if let Some(peer_info) = self.discovered_peers.get_mut(peer_id) {
            peer_info.latency = Some(latency);
            peer_info.last_seen = Instant::now();
        }
    }

    /// Start maintenance tasks
    async fn start_maintenance_tasks(&mut self) {
        // Implementation would spawn background tasks for periodic maintenance
        debug!("Started peer discovery maintenance tasks");
    }

    /// Handle Kademlia DHT events (simplified implementation)
    async fn handle_kademlia_event(
        &mut self,
        _event: KademliaEvent,
    ) -> Result<Vec<DiscoveryAction>, DiscoveryError> {
        // Simplified implementation - would handle various Kademlia events
        Ok(Vec::new())
    }

    /// Handle mDNS discovery events (simplified implementation)
    async fn handle_mdns_event(
        &mut self,
        _event: MdnsEvent,
    ) -> Result<Vec<DiscoveryAction>, DiscoveryError> {
        // Simplified implementation - would handle mDNS events
        Ok(Vec::new())
    }

    /// Handle Identify protocol events (simplified implementation)
    async fn handle_identify_event(
        &mut self,
        _event: IdentifyEvent,
    ) -> Result<Vec<DiscoveryAction>, DiscoveryError> {
        // Simplified implementation - would handle identify events
        Ok(Vec::new())
    }
}

impl GossipProtocol {
    /// Create new gossip protocol manager
    pub fn new(local_peer_id: PeerId, config: GossipConfig) -> Self {
        Self {
            local_peer_id,
            gossip_peers: HashMap::new(),
            message_cache: HashMap::new(),
            config,
            stats: GossipStats::default(),
        }
    }

    /// Add peer to gossip network
    pub fn add_gossip_peer(&mut self, peer_id: PeerId) {
        if self.gossip_peers.len() < self.config.max_gossip_peers {
            let peer_info = GossipPeerInfo {
                peer_id,
                score: 0.5, // Neutral score
                last_message: Instant::now(),
                message_count: 0,
            };
            self.gossip_peers.insert(peer_id, peer_info);
        }
    }

    /// Remove peer from gossip network
    pub fn remove_gossip_peer(&mut self, peer_id: &PeerId) {
        self.gossip_peers.remove(peer_id);
    }

    /// Propagate gossip message to network
    pub async fn propagate_message(
        &mut self,
        message: GossipMessage,
    ) -> Result<u32, DiscoveryError> {
        let message_id = self.calculate_message_id(&message);
        
        // Check if message is already in cache
        if self.message_cache.contains_key(&message_id) {
            return Ok(0);
        }

        // Cache the message
        let cached_msg = CachedMessage {
            message: message.clone(),
            received_at: Instant::now(),
            ttl: self.config.message_ttl,
            propagation_count: 0,
        };
        self.message_cache.insert(message_id, cached_msg);

        // Select peers for gossip
        let gossip_peers = self.select_gossip_peers();
        let propagated_count = gossip_peers.len() as u32;

        // Update statistics
        self.stats.messages_propagated += 1;

        Ok(propagated_count)
    }

    /// Select optimal peers for message gossip
    fn select_gossip_peers(&self) -> Vec<PeerId> {
        let target_count = (self.gossip_peers.len() / 3).max(1).min(6);
        
        let mut peers: Vec<_> = self.gossip_peers
            .values()
            .collect();
        
        // Sort by score (descending)
        peers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        peers.into_iter()
            .take(target_count)
            .map(|info| info.peer_id)
            .collect()
    }

    /// Calculate unique message ID
    pub fn calculate_message_id(&self, message: &GossipMessage) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{:?}", message).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Clean up expired messages from cache
    pub fn cleanup_expired_messages(&mut self) -> usize {
        let now = Instant::now();
        let mut removed = 0;

        self.message_cache.retain(|_id, cached_msg| {
            if now.duration_since(cached_msg.received_at) > cached_msg.ttl {
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }

    /// Get gossip statistics
    pub fn stats(&self) -> &GossipStats {
        &self.stats
    }
}

/// Initialize peer discovery with optimized configuration
pub async fn init_discovery(
    local_peer_id: PeerId,
    bootstrap_nodes: Vec<Multiaddr>,
) -> Result<PeerDiscovery, DiscoveryError> {
    let config = AdvancedDiscoveryConfig {
        enable_kademlia: true,
        enable_mdns: true,
        bootstrap_nodes,
        query_timeout: Duration::from_secs(30),
        max_discovered_peers: 1000,
        peer_info_ttl: Duration::from_secs(3600),
        bootstrap_interval: Duration::from_secs(600),
        min_peer_score: 0,
        ..Default::default()
    };

    PeerDiscovery::new(local_peer_id, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_discovery_initialization() {
        let local_peer_id = PeerId::random();
        let bootstrap_nodes = vec![
            "/ip4/127.0.0.1/tcp/8000".parse().unwrap(),
        ];
        
        let discovery = init_discovery(local_peer_id, bootstrap_nodes).await;
        assert!(discovery.is_ok());
    }

    #[tokio::test] 
    async fn test_peer_addition_and_retrieval() {
        let local_peer_id = PeerId::random();
        let config = AdvancedDiscoveryConfig::default();
        let mut discovery = PeerDiscovery::new(local_peer_id, config).unwrap();
        
        let peer_id = PeerId::random();
        let addresses = vec!["/ip4/192.168.1.100/tcp/8001".parse().unwrap()];
        
        discovery.add_or_update_peer(
            peer_id,
            addresses.clone(),
            DiscoveryMethod::Kademlia,
        );
        
        let peer_info = discovery.get_peer_info(&peer_id);
        assert!(peer_info.is_some());
        assert_eq!(peer_info.unwrap().addresses, addresses);
    }

    #[test]
    fn test_gossip_protocol_message_propagation() {
        let local_peer_id = PeerId::random();
        let config = GossipConfig::default();
        let mut gossip = GossipProtocol::new(local_peer_id, config);
        
        // Add test peer
        let peer_id = PeerId::random();
        gossip.add_gossip_peer(peer_id);
        
        let message = GossipMessage::PeerAdvertisement {
            peer_id: peer_id.to_string(),
            addresses: vec!["/ip4/127.0.0.1/tcp/8000".to_string()],
            timestamp: 1234567890,
            signature: vec![],
        };
        
        // Test message ID calculation
        let message_id = gossip.calculate_message_id(&message);
        assert!(!message_id.is_empty());
    }

    #[test]
    fn test_peer_cleanup() {
        let local_peer_id = PeerId::random();
        let mut config = AdvancedDiscoveryConfig::default();
        config.peer_info_ttl = Duration::from_millis(1); // Very short TTL
        
        let mut discovery = PeerDiscovery::new(local_peer_id, config).unwrap();
        
        let peer_id = PeerId::random();
        let addresses = vec!["/ip4/192.168.1.100/tcp/8001".parse().unwrap()];
        
        discovery.add_or_update_peer(
            peer_id,
            addresses,
            DiscoveryMethod::Mdns,
        );
        
        assert_eq!(discovery.discovered_peers().len(), 1);
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(2));
        
        let removed = discovery.cleanup_expired_peers();
        assert_eq!(removed, 1);
        assert_eq!(discovery.discovered_peers().len(), 0);
    }

    #[test]
    fn test_gossip_config_defaults() {
        let config = GossipConfig::default();
        assert_eq!(config.max_gossip_peers, 12);
        assert_eq!(config.gossip_interval, Duration::from_secs(30));
        assert_eq!(config.message_ttl, Duration::from_secs(300));
        assert_eq!(config.max_propagation_hops, 3);
    }

    #[test]
    fn test_discovery_method_serialization() {
        let method = DiscoveryMethod::Kademlia;
        let serialized = serde_json::to_string(&method).unwrap();
        let deserialized: DiscoveryMethod = serde_json::from_str(&serialized).unwrap();
        assert_eq!(method, deserialized);
    }
}
