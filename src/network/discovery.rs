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
    pub peer_id: PeerId,
    /// Multiaddresses for connections
    pub addresses: Vec<Multiaddr>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Agent version string
    pub agent_version: String,
    /// Protocol version
    pub protocol_version: String,
    /// Discovery timestamp
    pub discovered_at: Instant,
    /// Last seen timestamp
    pub last_seen: Instant,
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
                if let Ok(peer_id) = PeerId::from_multihash(peer_id_hash) {
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
}
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
