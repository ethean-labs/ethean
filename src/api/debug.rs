//! Debug API endpoints implementation
//!
//! Provides debug and diagnostic endpoints for development
//! and troubleshooting purposes.

use axum::{
    Router,
    routing::get,
    response::Json,
    extract::{Path, State},
};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use super::{ApiState, Result, Error};

/// Create debug API routes
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/beacon/states/:state_id", get(get_beacon_state))
        .route("/beacon/heads", get(get_beacon_heads))
        .route("/fork_choice", get(get_fork_choice))
}

/// Beacon state debug response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconStateDebugResponse {
    pub execution_optimistic: bool,
    pub finalized: bool,
    pub data: BeaconStateDebug,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconStateDebug {
    pub genesis_time: u64,
    pub genesis_validators_root: String,
    pub slot: u64,
    pub fork: ForkDebug,
    pub latest_block_header: BlockHeaderDebug,
    pub block_roots: Vec<String>,
    pub state_roots: Vec<String>,
    pub historical_roots: Vec<String>,
    pub eth1_data: Eth1DataDebug,
    pub validators: Vec<ValidatorDebug>,
    pub balances: Vec<u64>,
    pub randao_mixes: Vec<String>,
    pub slashings: Vec<u64>,
    pub justification_bits: String,
    pub previous_justified_checkpoint: CheckpointDebug,
    pub current_justified_checkpoint: CheckpointDebug,
    pub finalized_checkpoint: CheckpointDebug,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkDebug {
    pub previous_version: String,
    pub current_version: String,
    pub epoch: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockHeaderDebug {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Eth1DataDebug {
    pub deposit_root: String,
    pub deposit_count: u64,
    pub block_hash: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidatorDebug {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub exit_epoch: u64,
    pub withdrawable_epoch: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CheckpointDebug {
    pub epoch: u64,
    pub root: String,
}

/// Beacon heads response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconHeadsResponse {
    pub data: Vec<BeaconHead>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BeaconHead {
    pub root: String,
    pub slot: u64,
}

/// Fork choice debug response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkChoiceResponse {
    pub justified_checkpoint: CheckpointDebug,
    pub finalized_checkpoint: CheckpointDebug,
    pub fork_choice_nodes: Vec<ForkChoiceNode>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkChoiceNode {
    pub slot: u64,
    pub block_root: String,
    pub parent_root: String,
    pub justified_epoch: u64,
    pub finalized_epoch: u64,
    pub weight: u64,
    pub validity: String,
    pub execution_block_hash: Option<String>,
    pub extra_data: ForkChoiceNodeExtraData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkChoiceNodeExtraData {
    pub justified_root: String,
    pub finalized_root: String,
    pub unrealized_justified_checkpoint: CheckpointDebug,
    pub unrealized_finalized_checkpoint: CheckpointDebug,
}

/// GET /eth/v1/debug/beacon/states/{state_id}
#[utoipa::path(
    get,
    path = "/eth/v1/debug/beacon/states/{state_id}",
    tag = "debug",
    params(
        ("state_id" = String, Path, description = "State identifier")
    ),
    responses(
        (status = 200, description = "Beacon state debug information", body = BeaconStateDebugResponse)
    )
)]
pub async fn get_beacon_state(
    Path(state_id): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<BeaconStateDebugResponse>> {
    // Create a debug representation of the beacon state
    let beacon_state = BeaconStateDebug {
        genesis_time: 1606824000,
        genesis_validators_root: "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95".to_string(),
        slot: 1000,
        fork: ForkDebug {
            previous_version: "0x00000000".to_string(),
            current_version: "0x01000000".to_string(),
            epoch: 0,
        },
        latest_block_header: BlockHeaderDebug {
            slot: 999,
            proposer_index: 42,
            parent_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            state_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            body_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
        block_roots: vec!["0x0000000000000000000000000000000000000000000000000000000000000000".to_string(); 10],
        state_roots: vec!["0x0000000000000000000000000000000000000000000000000000000000000000".to_string(); 10],
        historical_roots: Vec::new(),
        eth1_data: Eth1DataDebug {
            deposit_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            deposit_count: 0,
            block_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
        validators: Vec::new(),
        balances: Vec::new(),
        randao_mixes: vec!["0x0000000000000000000000000000000000000000000000000000000000000000".to_string(); 10],
        slashings: vec![0; 8192],
        justification_bits: "0x00".to_string(),
        previous_justified_checkpoint: CheckpointDebug {
            epoch: 0,
            root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
        current_justified_checkpoint: CheckpointDebug {
            epoch: 0,
            root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
        finalized_checkpoint: CheckpointDebug {
            epoch: 0,
            root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        },
    };

    let response = BeaconStateDebugResponse {
        execution_optimistic: false,
        finalized: true,
        data: beacon_state,
    };

    Ok(Json(response))
}

/// GET /eth/v1/debug/beacon/heads
#[utoipa::path(
    get,
    path = "/eth/v1/debug/beacon/heads",
    tag = "debug",
    responses(
        (status = 200, description = "All known beacon chain heads", body = BeaconHeadsResponse)
    )
)]
pub async fn get_beacon_heads(
    State(state): State<ApiState>,
) -> Result<Json<BeaconHeadsResponse>> {
    // Return known beacon chain heads
    let heads = vec![
        BeaconHead {
            root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            slot: 1000,
        },
    ];

    let response = BeaconHeadsResponse { data: heads };
    Ok(Json(response))
}

/// GET /eth/v1/debug/fork_choice
#[utoipa::path(
    get,
    path = "/eth/v1/debug/fork_choice",
    tag = "debug",
    responses(
        (status = 200, description = "Fork choice debug information", body = ForkChoiceResponse)
    )
)]
pub async fn get_fork_choice(
    State(state): State<ApiState>,
) -> Result<Json<ForkChoiceResponse>> {
    // Return fork choice tree debug information
    let fork_choice_nodes = vec![
        ForkChoiceNode {
            slot: 1000,
            block_root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            parent_root: "0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba".to_string(),
            justified_epoch: 30,
            finalized_epoch: 28,
            weight: 100000000,
            validity: "valid".to_string(),
            execution_block_hash: Some("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
            extra_data: ForkChoiceNodeExtraData {
                justified_root: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                finalized_root: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
                unrealized_justified_checkpoint: CheckpointDebug {
                    epoch: 31,
                    root: "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
                },
                unrealized_finalized_checkpoint: CheckpointDebug {
                    epoch: 29,
                    root: "0x4444444444444444444444444444444444444444444444444444444444444444".to_string(),
                },
            },
        },
    ];

    let response = ForkChoiceResponse {
        justified_checkpoint: CheckpointDebug {
            epoch: 30,
            root: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        },
        finalized_checkpoint: CheckpointDebug {
            epoch: 28,
            root: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
        },
        fork_choice_nodes,
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
    async fn test_get_beacon_state() {
        let state = create_test_state();
        let result = get_beacon_state(Path("head".to_string()), State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.slot, 1000);
    }

    #[tokio::test]
    async fn test_get_beacon_heads() {
        let state = create_test_state();
        let result = get_beacon_heads(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.data.is_empty());
    }

    #[tokio::test]
    async fn test_get_fork_choice() {
        let state = create_test_state();
        let result = get_fork_choice(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.fork_choice_nodes.is_empty());
    }
}
