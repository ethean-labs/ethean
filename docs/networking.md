# Network Protocol Implementation

## Overview

Panro implements a robust P2P networking layer based on libp2p with custom protocol extensions for Ethereum Beacon Chain communication. The network layer handles peer discovery, connection management, message propagation, and bandwidth optimization.

## Network Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Network Layer                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Peer        │  │ Protocol    │  │ Bandwidth           │ │
│  │ Discovery   │  │ Handler     │  │ Management          │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Gossip      │  │ Connection  │  │ Message             │ │
│  │ Protocol    │  │ Pool        │  │ Validation          │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                      libp2p Transport                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ TCP/QUIC    │  │ Noise       │  │ Yamux               │ │
│  │ Transport   │  │ Encryption  │  │ Multiplexing        │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Protocol Specifications

### 1. Handshake Protocol

The handshake protocol establishes secure connections between peers and negotiates protocol versions.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub protocol_version: ProtocolVersion,
    pub client_version: String,
    pub chain_id: ChainId,
    pub genesis_validators_root: Root,
    pub capabilities: Vec<ProtocolCapability>,
    pub listen_addresses: Vec<Multiaddr>,
}

impl HandshakeProtocol {
    pub async fn initiate_handshake(
        &self,
        peer_id: PeerId,
    ) -> Result<HandshakeResponse, HandshakeError> {
        let message = HandshakeMessage {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            client_version: format!("Panro/{}", env!("CARGO_PKG_VERSION")),
            chain_id: self.config.chain_id,
            genesis_validators_root: self.config.genesis_validators_root,
            capabilities: vec![
                ProtocolCapability::BeaconBlocks,
                ProtocolCapability::BeaconAttestations,
                ProtocolCapability::SyncCommittee,
                ProtocolCapability::BlobSidecars,
            ],
            listen_addresses: self.swarm.listeners().cloned().collect(),
        };
        
        let response = self.send_handshake(peer_id, message).await?;
        self.validate_handshake_response(&response)?;
        
        Ok(response)
    }
    
    fn validate_handshake_response(
        &self,
        response: &HandshakeResponse,
    ) -> Result<(), HandshakeError> {
        // Validate protocol version compatibility
        if !self.is_protocol_compatible(&response.protocol_version) {
            return Err(HandshakeError::IncompatibleProtocol(
                response.protocol_version.clone()
            ));
        }
        
        // Validate chain compatibility
        if response.chain_id != self.config.chain_id {
            return Err(HandshakeError::IncompatibleChain(response.chain_id));
        }
        
        // Validate genesis
        if response.genesis_validators_root != self.config.genesis_validators_root {
            return Err(HandshakeError::IncompatibleGenesis(
                response.genesis_validators_root
            ));
        }
        
        Ok(())
    }
}
```

### 2. Message Protocol

#### Message Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    // Block-related messages
    BlocksByRange {
        start_slot: Slot,
        count: u64,
        step: u64,
    },
    BlocksByRoot {
        block_roots: Vec<Root>,
    },
    BeaconBlocks {
        blocks: Vec<SignedBeaconBlock>,
    },
    
    // Attestation messages
    Attestation {
        attestation: Attestation,
    },
    AggregateAndProof {
        aggregate: SignedAggregateAndProof,
    },
    
    // Sync committee messages
    SyncCommitteeMessage {
        message: SyncCommitteeMessage,
    },
    SyncCommitteeContribution {
        contribution: SignedContributionAndProof,
    },
    
    // Blob sidecar messages
    BlobSidecarsByRange {
        start_slot: Slot,
        count: u64,
    },
    BlobSidecarsByRoot {
        blob_ids: Vec<BlobIdentifier>,
    },
    BlobSidecars {
        sidecars: Vec<BlobSidecar>,
    },
    
    // Metadata and status
    Status {
        fork_digest: ForkDigest,
        finalized_root: Root,
        finalized_epoch: Epoch,
        head_root: Root,
        head_slot: Slot,
    },
    Ping {
        data: u64,
    },
    Pong {
        data: u64,
    },
    
    // Light client messages
    LightClientBootstrap {
        root: Root,
    },
    LightClientUpdatesByRange {
        start_period: u64,
        count: u64,
    },
    LightClientFinalityUpdate,
    LightClientOptimisticUpdate,
}
```

#### Message Encoding

```rust
pub struct MessageCodec {
    compression: CompressionType,
    max_message_size: usize,
}

impl MessageCodec {
    pub fn encode(&self, message: &NetworkMessage) -> Result<Bytes, CodecError> {
        // Serialize message
        let serialized = self.serialize_message(message)?;
        
        // Apply compression if enabled
        let compressed = if self.compression != CompressionType::None {
            self.compress_data(&serialized)?
        } else {
            serialized
        };
        
        // Check size limits
        if compressed.len() > self.max_message_size {
            return Err(CodecError::MessageTooLarge {
                size: compressed.len(),
                max_size: self.max_message_size,
            });
        }
        
        Ok(compressed.into())
    }
    
    pub fn decode(&self, data: &[u8]) -> Result<NetworkMessage, CodecError> {
        // Decompress if needed
        let decompressed = if self.compression != CompressionType::None {
            self.decompress_data(data)?
        } else {
            data.to_vec()
        };
        
        // Deserialize message
        self.deserialize_message(&decompressed)
    }
}
```

### 3. Gossip Protocol

The gossip protocol is used for broadcasting blocks, attestations, and other consensus messages.

```rust
pub struct GossipProtocol {
    swarm: Swarm<GossipBehaviour>,
    topics: HashMap<String, Topic>,
    message_validator: MessageValidator,
    duplicate_cache: LruCache<MessageId, ()>,
}

impl GossipProtocol {
    pub async fn publish_message(
        &mut self,
        topic: &str,
        message: NetworkMessage,
    ) -> Result<MessageId, GossipError> {
        let topic = self.topics.get(topic)
            .ok_or_else(|| GossipError::UnknownTopic(topic.to_string()))?;
        
        // Encode message
        let encoded = self.codec.encode(&message)?;
        
        // Publish to gossip network
        let message_id = self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), encoded)?;
        
        // Track published message
        self.published_messages.insert(message_id, message);
        
        Ok(message_id)
    }
    
    pub async fn handle_gossip_message(
        &mut self,
        peer_id: PeerId,
        message_id: MessageId,
        message: GossipsubMessage,
    ) -> Result<MessageAcceptance, GossipError> {
        // Check for duplicates
        if self.duplicate_cache.contains(&message_id) {
            return Ok(MessageAcceptance::Ignore);
        }
        
        // Decode message
        let network_message = self.codec.decode(&message.data)?;
        
        // Validate message
        let validation_result = self.message_validator
            .validate_message(peer_id, &network_message)
            .await?;
        
        match validation_result {
            ValidationResult::Accept => {
                // Process accepted message
                self.process_accepted_message(peer_id, network_message).await?;
                self.duplicate_cache.put(message_id, ());
                Ok(MessageAcceptance::Accept)
            }
            ValidationResult::Reject(reason) => {
                tracing::warn!("Rejected message from {}: {}", peer_id, reason);
                Ok(MessageAcceptance::Reject)
            }
            ValidationResult::Ignore => {
                Ok(MessageAcceptance::Ignore)
            }
        }
    }
}
```

### 4. Request-Response Protocol

For direct peer-to-peer communication and data synchronization.

```rust
pub struct RequestResponseProtocol {
    pending_requests: HashMap<RequestId, PendingRequest>,
    request_handlers: HashMap<String, Box<dyn RequestHandler>>,
    rate_limiter: RateLimiter,
}

impl RequestResponseProtocol {
    pub async fn send_request<T>(
        &mut self,
        peer_id: PeerId,
        request: T,
    ) -> Result<T::Response, RequestError>
    where
        T: Request + Send + 'static,
        T::Response: Response + Send + 'static,
    {
        // Check rate limits
        if !self.rate_limiter.check_request_allowed(peer_id) {
            return Err(RequestError::RateLimited);
        }
        
        // Generate request ID
        let request_id = RequestId::new();
        
        // Encode request
        let encoded_request = self.encode_request(&request)?;
        
        // Send request
        self.swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, encoded_request);
        
        // Create pending request
        let (tx, rx) = oneshot::channel();
        let pending = PendingRequest {
            peer_id,
            request_type: std::any::type_name::<T>().to_string(),
            timestamp: Instant::now(),
            response_sender: tx,
        };
        
        self.pending_requests.insert(request_id, pending);
        
        // Wait for response with timeout
        let response = tokio::time::timeout(
            self.config.request_timeout,
            rx,
        ).await??;
        
        // Decode response
        self.decode_response(response)
    }
    
    pub fn register_handler<T>(&mut self, handler: T)
    where
        T: RequestHandler + Send + Sync + 'static,
    {
        let protocol_name = T::protocol_name();
        self.request_handlers.insert(protocol_name, Box::new(handler));
    }
}
```

## Peer Discovery

### Discovery Mechanisms

```rust
pub struct PeerDiscovery {
    mdns: Option<Mdns>,
    kademlia: Kademlia<MemoryStore>,
    bootstrap_nodes: Vec<Multiaddr>,
    discovered_peers: HashSet<PeerId>,
}

impl PeerDiscovery {
    pub async fn start_discovery(&mut self) -> Result<(), DiscoveryError> {
        // Start mDNS discovery for local peers
        if let Some(mdns) = &mut self.mdns {
            mdns.reset();
        }
        
        // Bootstrap Kademlia DHT
        for addr in &self.bootstrap_nodes {
            if let Ok(peer_id) = addr.extract_peer_id() {
                self.kademlia.add_address(&peer_id, addr.clone());
            }
        }
        
        // Start DHT bootstrap
        self.kademlia.bootstrap()?;
        
        Ok(())
    }
    
    pub async fn handle_discovery_event(
        &mut self,
        event: DiscoveryEvent,
    ) -> Result<(), DiscoveryError> {
        match event {
            DiscoveryEvent::MdnsDiscovered { peer_id, addresses } => {
                tracing::info!("Discovered peer via mDNS: {}", peer_id);
                for addr in addresses {
                    self.kademlia.add_address(&peer_id, addr);
                }
                self.discovered_peers.insert(peer_id);
            }
            DiscoveryEvent::KademliaDiscovered { peer_id, addresses } => {
                tracing::info!("Discovered peer via Kademlia: {}", peer_id);
                for addr in addresses {
                    self.swarm.dial(addr)?;
                }
                self.discovered_peers.insert(peer_id);
            }
            DiscoveryEvent::BootstrapCompleted => {
                tracing::info!("DHT bootstrap completed");
                self.start_periodic_queries().await?;
            }
        }
        
        Ok(())
    }
}
```

## Connection Management

### Connection Pool

```rust
pub struct ConnectionPool {
    connections: HashMap<PeerId, Connection>,
    connection_limits: ConnectionLimits,
    health_checker: HealthChecker,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub peer_id: PeerId,
    pub address: Multiaddr,
    pub connection_time: Instant,
    pub last_activity: Instant,
    pub direction: ConnectionDirection,
    pub status: ConnectionStatus,
    pub bandwidth_stats: BandwidthStats,
}

impl ConnectionPool {
    pub async fn establish_connection(
        &mut self,
        peer_id: PeerId,
        address: Multiaddr,
    ) -> Result<(), ConnectionError> {
        // Check connection limits
        if self.connections.len() >= self.connection_limits.max_connections {
            self.evict_oldest_connection().await?;
        }
        
        // Attempt connection
        let connection = self.connect_to_peer(peer_id, address).await?;
        
        // Add to pool
        self.connections.insert(peer_id, connection);
        
        // Start health monitoring
        self.health_checker.start_monitoring(peer_id).await?;
        
        Ok(())
    }
    
    pub async fn handle_connection_event(
        &mut self,
        event: ConnectionEvent,
    ) -> Result<(), ConnectionError> {
        match event {
            ConnectionEvent::Established { peer_id, address, direction } => {
                let connection = Connection {
                    peer_id,
                    address,
                    connection_time: Instant::now(),
                    last_activity: Instant::now(),
                    direction,
                    status: ConnectionStatus::Connected,
                    bandwidth_stats: BandwidthStats::default(),
                };
                
                self.connections.insert(peer_id, connection);
                tracing::info!("Connection established with {}", peer_id);
            }
            ConnectionEvent::Closed { peer_id, reason } => {
                self.connections.remove(&peer_id);
                self.health_checker.stop_monitoring(peer_id).await?;
                tracing::info!("Connection closed with {}: {:?}", peer_id, reason);
            }
            ConnectionEvent::Error { peer_id, error } => {
                if let Some(connection) = self.connections.get_mut(&peer_id) {
                    connection.status = ConnectionStatus::Error(error.clone());
                }
                tracing::warn!("Connection error with {}: {}", peer_id, error);
            }
        }
        
        Ok(())
    }
}
```

## Bandwidth Management

### Rate Limiting

```rust
pub struct NetworkRateLimiter {
    global_limiter: TokenBucket,
    peer_limiters: HashMap<PeerId, TokenBucket>,
    bandwidth_monitor: BandwidthMonitor,
}

impl NetworkRateLimiter {
    pub async fn check_send_allowed(
        &mut self,
        peer_id: PeerId,
        message_size: usize,
    ) -> Result<bool, RateLimitError> {
        // Check global rate limit
        if !self.global_limiter.try_consume(message_size as u64).await {
            return Ok(false);
        }
        
        // Check per-peer rate limit
        let peer_limiter = self.peer_limiters
            .entry(peer_id)
            .or_insert_with(|| TokenBucket::new(
                self.config.per_peer_rate_limit,
                self.config.per_peer_burst_limit,
            ));
        
        if !peer_limiter.try_consume(message_size as u64).await {
            // Refund global tokens
            self.global_limiter.add_tokens(message_size as u64).await;
            return Ok(false);
        }
        
        // Record bandwidth usage
        self.bandwidth_monitor.record_outbound(peer_id, message_size).await;
        
        Ok(true)
    }
    
    pub async fn on_message_received(
        &mut self,
        peer_id: PeerId,
        message_size: usize,
    ) -> Result<(), RateLimitError> {
        // Record inbound bandwidth
        self.bandwidth_monitor.record_inbound(peer_id, message_size).await;
        
        // Check if peer is exceeding limits
        let peer_stats = self.bandwidth_monitor.get_peer_stats(peer_id).await;
        
        if peer_stats.inbound_rate > self.config.max_peer_inbound_rate {
            // Apply penalties or disconnect
            self.apply_rate_limit_penalty(peer_id).await?;
        }
        
        Ok(())
    }
}
```

## Security and Validation

### Message Validation

```rust
pub struct MessageValidator {
    signature_verifier: SignatureVerifier,
    spam_detector: SpamDetector,
    peer_reputation: PeerReputation,
}

impl MessageValidator {
    pub async fn validate_message(
        &mut self,
        peer_id: PeerId,
        message: &NetworkMessage,
    ) -> Result<ValidationResult, ValidationError> {
        // Check peer reputation
        if self.peer_reputation.is_blacklisted(peer_id) {
            return Ok(ValidationResult::Reject("Blacklisted peer".to_string()));
        }
        
        // Basic structure validation
        self.validate_message_structure(message)?;
        
        // Spam detection
        if self.spam_detector.is_spam(peer_id, message).await? {
            return Ok(ValidationResult::Reject("Spam detected".to_string()));
        }
        
        // Message-specific validation
        match message {
            NetworkMessage::BeaconBlocks { blocks } => {
                for block in blocks {
                    self.validate_beacon_block(block).await?;
                }
            }
            NetworkMessage::Attestation { attestation } => {
                self.validate_attestation(attestation).await?;
            }
            NetworkMessage::AggregateAndProof { aggregate } => {
                self.validate_aggregate_and_proof(aggregate).await?;
            }
            _ => {}
        }
        
        // Update peer reputation on successful validation
        self.peer_reputation.record_valid_message(peer_id);
        
        Ok(ValidationResult::Accept)
    }
    
    async fn validate_beacon_block(
        &self,
        block: &SignedBeaconBlock,
    ) -> Result<(), ValidationError> {
        // Verify block signature
        if !self.signature_verifier
            .verify_block_signature(block)
            .await?
        {
            return Err(ValidationError::InvalidSignature);
        }
        
        // Validate block structure
        if block.message.body.attestations.len() > MAX_ATTESTATIONS {
            return Err(ValidationError::TooManyAttestations);
        }
        
        // Additional validation logic...
        
        Ok(())
    }
}
```

## Configuration

### Network Configuration

```toml
[network]
# Basic network settings
listen_address = "0.0.0.0:9000"
discovery_port = 9001
max_peers = 100
target_peers = 50

# Protocol settings
protocol_version = "1.0.0"
client_version = "Panro/0.1.0"
user_agent = "Panro Beacon Chain Client"

# Connection settings
connection_timeout = "10s"
handshake_timeout = "5s"
max_concurrent_dials = 10
dial_concurrency_factor = 8

# Message settings
max_message_size = 1048576  # 1MB
compression_enabled = true
compression_level = 6

# Rate limiting
global_rate_limit_bps = 10485760  # 10MB/s
per_peer_rate_limit_bps = 1048576  # 1MB/s
burst_limit_multiplier = 2.0

# Gossip settings
[network.gossip]
mesh_n = 6
mesh_n_low = 5
mesh_n_high = 12
gossip_lazy = 6
heartbeat_interval = "1s"
fanout_ttl = "60s"

# Discovery settings
[network.discovery]
enable_mdns = true
enable_kademlia = true
bootstrap_nodes = [
    "/dns4/bootnode1.example.com/tcp/9000/p2p/16Uiu2HAm...",
    "/dns4/bootnode2.example.com/tcp/9000/p2p/16Uiu2HAm...",
]
kademlia_replication_factor = 20

# Security settings
[network.security]
enable_noise_encryption = true
enable_peer_scoring = true
blacklist_threshold = -100
whitelist_persistence = "24h"
```
