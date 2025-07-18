//! P2P Network Layer for Beam Chain
//!
//! Implements libp2p-based networking with Gossipsub message propagation,
//! peer discovery, and consensus message handling.

pub mod peer_manager;
pub mod gossip;
pub mod discovery;
pub mod message_handler;
pub mod network_config;

use crate::types::{BeaconBlock, Attestation, Epoch, Slot};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Main network manager
#[derive(Debug)]
pub struct NetworkManager {
    peer_manager: Arc<peer_manager::PeerManager>,
    gossip: Arc<gossip::GossipService>,
    discovery: Arc<discovery::Discovery>,
    message_handler: Arc<message_handler::MessageHandler>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new() -> Self {
        let peer_manager = Arc::new(peer_manager::PeerManager::new());
        let gossip = Arc::new(gossip::GossipService::new());
        let discovery = Arc::new(discovery::Discovery::new());
        let message_handler = Arc::new(message_handler::MessageHandler::new());
        
        Self {
            peer_manager,
            gossip,
            discovery,
            message_handler,
        }
    }
    
    /// Start the network manager
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start network services
        Ok(())
    }
}
use thiserror::Error;

// Re-export main components
pub use peer_manager::{PeerManager, PeerState, PeerScore};
pub use gossip::{GossipService, GossipMessage};
pub use discovery::{DiscoveryService, DiscoveryNode};
pub use message_handler::{MessageHandler, MessageResult, MessagePriority};
pub use network_config::{NetworkConfig, PeerConfig};

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
}

/// Network message types for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Beacon block propagation
    BeaconBlock(BeaconBlock),
    /// Attestation propagation  
    Attestation(Attestation),
    /// Block request/response
    BlockRequest { slot: Slot, count: u32 },
    BlockResponse { blocks: Vec<BeaconBlock> },
    /// Status sync messages
    StatusRequest,
    StatusResponse { head_slot: Slot, finalized_epoch: Epoch },
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
