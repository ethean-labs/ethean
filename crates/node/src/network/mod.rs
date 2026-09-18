//! P2P Network Layer for Beam Chain
//!
//! Implements libp2p-based networking with Gossipsub message propagation,
//! peer discovery, and consensus message handling.

pub mod peer_manager;
pub mod gossip;
pub mod discovery;
pub mod message_handler;
pub mod network_config;
pub mod bandwidth;
pub mod protocol;
pub mod connection_manager;
pub mod security;
pub mod performance;
pub mod orchestrator;

use ethean_types::{Attestation, Block, Slot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Main network manager
#[derive(Debug)]
pub struct NetworkManager {
    // Simplified network manager without complex dependencies
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new() -> Self {
        Self {}
    }
    
    /// Start the network manager
    pub async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start network services
        Ok(())
    }
}
use thiserror::Error;

// Re-export main components
pub use peer_manager::{PeerManager, PeerState, PeerScore};
pub use gossip::{GossipService, GossipMessage};
pub use discovery::{DiscoveryService, DiscoveryNode, PeerDiscovery, AdvancedDiscoveryConfig};
pub use message_handler::{MessageHandler, MessageResult, MessagePriority};
pub use network_config::{NetworkConfig, PeerConfig};
pub use connection_manager::{ConnectionPool, ConnectionInfo, ConnectionPoolConfig};
pub use security::{NetworkSecurity, SecurityConfig, AuthMethod, TrustLevel};
pub use performance::{PerformanceOptimizer, PerformanceConfig, PerformanceMetrics};
pub use orchestrator::{NetworkOrchestrator, OrchestratorBuilder, OrchestratorConfig, NetworkStatus};

/// Network errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Peer connection failed: {peer_id}")]
    ConnectionFailed { peer_id: String },
    
    #[error("Message validation failed: {reason}")]
    MessageValidation { reason: String },
    
    #[error("Network configuration error: {details}")]
    Configuration { details: String },
    
    #[error("Gossip propagation failed: {topic}")]
    GossipFailed { topic: String },
    
    #[error("Discovery error: {reason}")]
    Discovery { reason: String },

    #[error("Component initialization failed: {component} - {reason}")]
    InitializationFailed { component: String, reason: String },

    #[error("Network startup failed: {component} - {reason}")]
    StartupFailed { component: String, reason: String },

    #[error("Security violation: peer {peer_id} - {reason}")]
    SecurityViolation { peer_id: String, reason: String },

    #[error("Invalid network state: current={current}, expected={expected}")]
    InvalidState { current: String, expected: String },

    #[error("Performance optimization failed: {reason}")]
    PerformanceError { reason: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Rate limiting error: {reason}")]
    RateLimitError { reason: String },
}

/// Network message types for consensus (no Serde on Lean containers).
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Block propagation
    Block(Block),
    /// Attestation propagation
    Attestation(Attestation),
    /// Block request/response
    BlockRequest { slot: Slot, count: u32 },
    BlockResponse { blocks: Vec<Block> },
    /// Status sync messages
    StatusRequest,
    StatusResponse { head_slot: Slot, finalized_slot: Slot },
}

/// Peer information and scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub connected_at: u64,
    pub last_seen: u64,
    pub score: i32,
    pub user_agent: String,
    pub protocols: Vec<String>,
}

/// Network performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: u32,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub average_latency_ms: f64,
    pub gossip_topics: HashMap<String, u64>,
}

/// Legacy network result type for compatibility
pub type Result<T> = std::result::Result<T, NetworkError>;

/// Legacy error type for compatibility  
pub type Error = NetworkError;
