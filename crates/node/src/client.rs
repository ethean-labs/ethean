//! Main Ethean client implementation

use crate::{
    api::{ApiConfig, ApiServer},
    config::Config,
    consensus::validator_management::{ValidatorConfig, ValidatorManager},
    storage::{database::Database, StateStore},
};
use ethean_profile::ChainProfile;
use std::sync::Arc;
use tracing::{error, info};

/// Main result type for client operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main Ethean client
pub struct EtheanClient {
    _config: Config,
    _profile: ChainProfile,
    api_server: ApiServer,
}

impl EtheanClient {
    /// Create a new Ethean client
    pub async fn new() -> Result<Self> {
        let config = Config::default();
        let profile = config.chain_profile()?;

        info!(
            fork = profile.fork_name,
            seconds_per_slot = profile.seconds_per_slot,
            "Loaded chain profile"
        );

        let database = Database::in_memory();
        let state_store = Arc::new(StateStore::new(database));

        let validator_config = ValidatorConfig {
            min_deposit_amount: 32_000_000_000,
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

        let api_config = ApiConfig {
            bind_addr: config.api.bind_addr,
            max_request_size: 1024 * 1024,
            enable_cors: true,
            enable_compression: true,
            rate_limit: crate::api::RateLimitConfig::default(),
        };

        let api_server = ApiServer::new(validator_manager, state_store, api_config);

        Ok(Self {
            _config: config,
            _profile: profile,
            api_server,
        })
    }

    /// Start the client
    pub async fn start(self) -> Result<()> {
        info!("Starting Ethean client");

        if let Err(e) = self.api_server.start().await {
            error!("Failed to start API server: {}", e);
            return Err(e);
        }

        Ok(())
    }
}
