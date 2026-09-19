//! Config API endpoints — Lean profile surface (not Beacon 12s / 32-slot epochs).

use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::error::{Error, Result};
use super::ApiState;
use ethean_profile::{lstar_devnet, ForkId};

/// Create config API routes
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/fork_schedule", get(get_fork_schedule))
        .route("/spec", get(get_spec))
        .route("/deposit_contract", get(get_deposit_contract))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkScheduleResponse {
    pub data: Vec<ForkInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ForkInfo {
    /// Human fork name only; 4-byte digest unresolved (Phase 00).
    pub fork_name: String,
    pub epoch: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecResponse {
    pub data: SpecData,
}

/// Lean consensus timing/bounds from the pinned profile.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SpecData {
    pub genesis_slot: u64,
    pub fork_name: String,
    pub seconds_per_slot: u64,
    pub intervals_per_slot: u64,
    pub milliseconds_per_slot: u64,
    pub milliseconds_per_interval: u64,
    pub justification_lookback_slots: u64,
    pub validator_registry_limit: u64,
    pub historical_roots_limit: u64,
    pub max_attestations_data: u64,
    pub xmss_public_key_bytes: u64,
    pub xmss_signature_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositContractResponse {
    pub data: DepositContractData,
}

/// Operational placeholder — not protocol timing truth.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DepositContractData {
    pub note: String,
    pub chain_id: Option<u64>,
    pub address: Option<String>,
}

#[utoipa::path(
    get,
    path = "/eth/v1/config/fork_schedule",
    tag = "config",
    responses((status = 200, description = "Fork schedule", body = ForkScheduleResponse))
)]
pub async fn get_fork_schedule(
    State(_state): State<ApiState>,
) -> Result<Json<ForkScheduleResponse>> {
    let response = ForkScheduleResponse {
        data: vec![ForkInfo {
            fork_name: ForkId::lstar().fork_name().to_string(),
            epoch: 0,
        }],
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/eth/v1/config/spec",
    tag = "config",
    responses((status = 200, description = "Chain specification", body = SpecResponse))
)]
pub async fn get_spec(State(_state): State<ApiState>) -> Result<Json<SpecResponse>> {
    let profile = lstar_devnet().map_err(|e| Error::internal_server_error(e.to_string()))?;
    let spec = SpecData {
        genesis_slot: 0,
        fork_name: profile.fork_name.to_string(),
        seconds_per_slot: profile.seconds_per_slot,
        intervals_per_slot: profile.intervals_per_slot,
        milliseconds_per_slot: profile.milliseconds_per_slot,
        milliseconds_per_interval: profile.milliseconds_per_interval,
        justification_lookback_slots: profile.justification_lookback_slots,
        validator_registry_limit: profile.validator_registry_limit,
        historical_roots_limit: profile.historical_roots_limit,
        max_attestations_data: profile.max_attestations_data,
        xmss_public_key_bytes: profile.xmss_public_key_bytes,
        xmss_signature_bytes: profile.xmss_signature_bytes,
    };
    Ok(Json(SpecResponse { data: spec }))
}

#[utoipa::path(
    get,
    path = "/eth/v1/config/deposit_contract",
    tag = "config",
    responses((status = 200, description = "Deposit contract information", body = DepositContractResponse))
)]
pub async fn get_deposit_contract(
    State(_state): State<ApiState>,
) -> Result<Json<DepositContractResponse>> {
    Ok(Json(DepositContractResponse {
        data: DepositContractData {
            note: "Lean Consensus has no Beacon mainnet deposit-contract pin in Phase 04".into(),
            chain_id: None,
            address: None,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{Database, StateStore};
    use std::sync::Arc;

    fn create_test_state() -> ApiState {
        let validator_config = ValidatorConfig::default();
        let state_store = Arc::new(StateStore::new(Database::in_memory()));
        let validator_manager =
            Arc::new(ValidatorManager::new(validator_config, (*state_store).clone()));
        let config = super::super::ApiConfig::default();
        ApiState {
            validator_manager,
            state_store,
            config,
        }
    }

    #[tokio::test]
    async fn test_get_fork_schedule() {
        let result = get_fork_schedule(State(create_test_state())).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().data[0].fork_name, "lstar");
    }

    #[tokio::test]
    async fn test_get_spec_lean_timing() {
        let response = get_spec(State(create_test_state())).await.unwrap();
        assert_eq!(response.data.seconds_per_slot, 4);
        assert_eq!(response.data.intervals_per_slot, 5);
        assert_eq!(response.data.milliseconds_per_slot, 4000);
    }

    #[tokio::test]
    async fn test_get_deposit_contract_no_mainnet() {
        let response = get_deposit_contract(State(create_test_state()))
            .await
            .unwrap();
        assert!(response.data.chain_id.is_none());
        assert!(response.data.address.is_none());
    }
}
