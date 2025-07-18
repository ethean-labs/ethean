//! Config API endpoints implementation
//!
//! Provides access to beacon chain configuration parameters
//! and specification constants.

use axum::{
    Router,
    routing::get,
    response::Json,
    extract::State,
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use super::{ApiState, Result, Error};

/// Create config API routes
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/fork_schedule", get(get_fork_schedule))
        .route("/spec", get(get_spec))
        .route("/deposit_contract", get(get_deposit_contract))
}

/// Fork schedule response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkScheduleResponse {
    pub data: Vec<ForkInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkInfo {
    pub previous_version: [u8; 4],
    pub current_version: [u8; 4],
    pub epoch: u64,
}

/// Spec response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecResponse {
    pub data: SpecData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecData {
    // Core constants
    pub genesis_slot: u64,
    pub genesis_epoch: u64,
    pub far_future_epoch: u64,
    pub base_rewards_per_epoch: u64,
    
    // Time parameters
    pub seconds_per_slot: u64,
    pub slots_per_epoch: u64,
    pub epochs_per_eth1_voting_period: u64,
    pub epochs_per_historical_vector: u64,
    pub epochs_per_slashings_vector: u64,
    
    // Validator parameters
    pub max_committees_per_slot: u64,
    pub target_committee_size: u64,
    pub max_validators_per_committee: u64,
    pub shuffle_round_count: u64,
    
    // Ethereum 1 parameters
    pub eth1_follow_distance: u64,
    pub max_request_blocks: u64,
    
    // Gwei values
    pub min_deposit_amount: u64,
    pub max_effective_balance: u64,
    pub effective_balance_increment: u64,
}

/// Deposit contract response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositContractResponse {
    pub data: DepositContractData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositContractData {
    pub chain_id: u64,
    pub address: String,
}

/// GET /eth/v1/config/fork_schedule
#[utoipa::path(
    get,
    path = "/eth/v1/config/fork_schedule",
    tag = "config",
    responses(
        (status = 200, description = "Fork schedule", body = ForkScheduleResponse)
    )
)]
pub async fn get_fork_schedule(
    State(state): State<ApiState>,
) -> Result<Json<ForkScheduleResponse>> {
    let forks = vec![
        ForkInfo {
            previous_version: [0, 0, 0, 0],
            current_version: [1, 0, 0, 0],
            epoch: 0,
        },
        // Add more forks as needed
    ];

    let response = ForkScheduleResponse { data: forks };
    Ok(Json(response))
}

/// GET /eth/v1/config/spec
#[utoipa::path(
    get,
    path = "/eth/v1/config/spec",
    tag = "config",
    responses(
        (status = 200, description = "Chain specification", body = SpecResponse)
    )
)]
pub async fn get_spec(
    State(state): State<ApiState>,
) -> Result<Json<SpecResponse>> {
    let spec = SpecData {
        // Core constants
        genesis_slot: 0,
        genesis_epoch: 0,
        far_future_epoch: u64::MAX,
        base_rewards_per_epoch: 4,
        
        // Time parameters
        seconds_per_slot: 12,
        slots_per_epoch: 32,
        epochs_per_eth1_voting_period: 64,
        epochs_per_historical_vector: 65536,
        epochs_per_slashings_vector: 8192,
        
        // Validator parameters
        max_committees_per_slot: 64,
        target_committee_size: 128,
        max_validators_per_committee: 2048,
        shuffle_round_count: 90,
        
        // Ethereum 1 parameters
        eth1_follow_distance: 2048,
        max_request_blocks: 1024,
        
        // Gwei values (in Gwei)
        min_deposit_amount: 1_000_000_000, // 1 ETH
        max_effective_balance: 32_000_000_000, // 32 ETH
        effective_balance_increment: 1_000_000_000, // 1 ETH
    };

    let response = SpecResponse { data: spec };
    Ok(Json(response))
}

/// GET /eth/v1/config/deposit_contract
#[utoipa::path(
    get,
    path = "/eth/v1/config/deposit_contract",
    tag = "config",
    responses(
        (status = 200, description = "Deposit contract information", body = DepositContractResponse)
    )
)]
pub async fn get_deposit_contract(
    State(state): State<ApiState>,
) -> Result<Json<DepositContractResponse>> {
    let deposit_contract = DepositContractData {
        chain_id: 1, // Mainnet
        address: "0x00000000219ab540356cBB839Cbe05303d7705Fa".to_string(),
    };

    let response = DepositContractResponse {
        data: deposit_contract,
    };
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    use crate::network::{NetworkService, NetworkConfig};
    use std::sync::Arc;

    fn create_test_state() -> ApiState {
        let validator_config = ValidatorConfig::default();
        let state_store = Arc::new(StateStore::new(Database::in_memory()));
        let validator_manager = Arc::new(ValidatorManager::new(validator_config, state_store.clone()));
        let network_config = NetworkConfig::local();
        let network_service = Arc::new(NetworkService::new(network_config).unwrap());
        let config = super::super::ApiConfig::default();
        
        ApiState {
            validator_manager,
            state_store,
            network_service,
            config,
        }
    }

    #[tokio::test]
    async fn test_get_fork_schedule() {
        let state = create_test_state();
        let result = get_fork_schedule(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.data.is_empty());
    }

    #[tokio::test]
    async fn test_get_spec() {
        let state = create_test_state();
        let result = get_spec(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.seconds_per_slot, 12);
        assert_eq!(response.data.slots_per_epoch, 32);
    }

    #[tokio::test]
    async fn test_get_deposit_contract() {
        let state = create_test_state();
        let result = get_deposit_contract(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.chain_id, 1);
        assert!(!response.data.address.is_empty());
    }
}
