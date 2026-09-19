//! Main Ethean client implementation

use crate::{
    api::{ApiConfig, ApiServer},
    clock::{clock_from_genesis, SlotClock},
    config::Config,
    consensus::validator_management::{ValidatorConfig, ValidatorManager},
    storage::{database::Database, StateStore},
};
use ethean_genesis::{local_smoke_genesis, BuiltGenesis, GenesisBuilder, GenesisError};
use ethean_profile::{lstar_devnet, require_lstar_fork, ChainProfile};
use ethean_types::State;
use std::sync::Arc;
use tracing::{error, info};

/// Main result type for client operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main Ethean client
pub struct EtheanClient {
    _config: Config,
    _profile: ChainProfile,
    _genesis: State,
    _clock: SlotClock,
    api_server: ApiServer,
}

impl EtheanClient {
    /// Start with an explicit pinned profile and verified genesis state.
    ///
    /// Does **not** use `State::default()` as network genesis.
    pub async fn with_genesis(profile: ChainProfile, genesis: State) -> Result<Self> {
        require_lstar_fork(&profile)?;
        if genesis.validators.is_empty() {
            return Err(Box::new(GenesisError::EmptyValidators));
        }

        let clock = clock_from_genesis(&genesis, profile.clone())?;
        let config = Config::default();

        info!(
            fork = profile.fork_name,
            seconds_per_slot = profile.seconds_per_slot,
            genesis_time = genesis.genesis_time(),
            validators = genesis.validators.len(),
            "Loaded chain profile and genesis"
        );

        let database = Database::in_memory();
        let state_store = Arc::new(StateStore::new(database));

        let validator_config = ValidatorConfig {
            max_validators: profile.validator_registry_limit,
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
            _genesis: genesis,
            _clock: clock,
            api_server,
        })
    }

    /// Build genesis via [`GenesisBuilder`], then start.
    pub async fn from_builder(profile: ChainProfile, builder: GenesisBuilder) -> Result<Self> {
        let BuiltGenesis { state, state_root } = builder.build()?;
        info!(?state_root, "Genesis built");
        Self::with_genesis(profile, state).await
    }

    /// Local smoke start: `lstar_devnet` + single zero-key validator genesis.
    pub async fn new() -> Result<Self> {
        let profile = lstar_devnet()?;
        let built = local_smoke_genesis(1_700_000_000)?;
        Self::with_genesis(profile, built.state).await
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
