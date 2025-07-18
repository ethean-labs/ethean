//! Main Panro client implementation

use crate::{
    api::ApiServer,
    config::Config,
    consensus::validator_management::ValidatorManager,
    storage::StateStore,
    network::NetworkManager,
};
use std::sync::Arc;
use tracing::{info, error};

/// Main result type for client operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main Panro client
pub struct PanroClient {
    _config: Config,
    api_server: ApiServer,
}

impl PanroClient {
    /// Create a new Panro client
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config = Config::default();
        
        // Create validator manager
        let validator_manager = Arc::new(ValidatorManager::new());
        
        // Create state store
        let state_store = Arc::new(StateStore::new());
        
        // Create network manager
        let network_manager = Arc::new(NetworkManager::new());
        
        // Create API server
        let api_server = ApiServer::new(
            config.clone(),
            validator_manager,
            state_store,
            network_manager,
        );
        
        Ok(Self {
            _config: config,
            api_server,
        })
    }
    
    /// Start the client
    pub async fn start(self) -> Result<()> {
        info!("Starting Panro client");
        
        // Start API server
        if let Err(e) = self.api_server.start().await {
            error!("Failed to start API server: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
}
