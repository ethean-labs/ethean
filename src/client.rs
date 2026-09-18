//! Main Ethean client implementation

use crate::{
    api::{ApiServer, ApiConfig},
    config::Config,
    consensus::validator_management::{ValidatorManager, ValidatorConfig},
    storage::{StateStore, database::Database},
};
use std::sync::Arc;
use tracing::{info, error};

/// Main result type for client operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main Ethean client
pub struct EtheanClient {
    _config: Config,
    api_server: ApiServer,
}

impl EtheanClient {
    /// Create a new Ethean client
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config = Config::default();
        
        // Create database
        let database = Database::in_memory();
        
        // Create state store
        let state_store = Arc::new(StateStore::new(database));
        
        // Create validator manager with proper config
        let validator_config = ValidatorConfig {
            min_deposit_amount: 32_000_000_000,  // 32 ETH in Gwei
            max_validators_per_epoch: 1000,
            activation_delay: 4,
            exit_delay: 256,
            slashing_penalty_multiplier: 3,
            inactivity_penalty_per_epoch: 1_000_000,
        };
        let validator_manager = Arc::new(ValidatorManager::new(
            validator_config,
            (*state_store).clone(),
        ));
        
        // Create API config
        let api_config = ApiConfig {
            bind_addr: config.api.bind_addr,
            max_request_size: 1024 * 1024, // 1MB
            enable_cors: true,
            enable_compression: true,
            rate_limit: crate::api::RateLimitConfig::default(),
        };
        
        // Create API server
        let api_server = ApiServer::new(
            validator_manager,
            state_store,
            api_config,
        );
        
        Ok(Self {
            _config: config,
            api_server,
        })
    }
    
    /// Start the client
    pub async fn start(self) -> Result<()> {
        info!("Starting Ethean client");
        
        // Start API server
        if let Err(e) = self.api_server.start().await {
            error!("Failed to start API server: {}", e);
            return Err(e);
        }
        
        Ok(())
    }
}
