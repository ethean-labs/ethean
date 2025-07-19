//! Main Panro client implementation

use crate::{
    api::ApiServer,
    config::{Config, ApiConfig},
    consensus::validator_management::{ValidatorManager, ValidatorConfig},
    storage::{StateStore, database::Database},
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
        
        // Create database
        let database = Database::in_memory();
        
        // Create state store
        let state_store = Arc::new(StateStore::new(database));
        
        // Create validator manager with proper config
        let validator_config = ValidatorConfig {
            max_validators: 1000,
            slashing_protection_enabled: true,
            keystore_path: "./keystores".to_string(),
        };
        let validator_manager = Arc::new(ValidatorManager::new(
            validator_config,
            (*state_store).clone(),
        ));
        
        // Create API server
        let api_server = ApiServer::new(
            validator_manager,
            state_store,
            config.api.clone(),
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
