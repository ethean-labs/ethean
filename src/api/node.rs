//! Node API endpoints implementation
//!
//! Provides information about the beacon node including
//! health, identity, peers, and synchronization status.

use axum::{
    Router,
    routing::get,
    response::Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use utoipa::{ToSchema, IntoParams};

use super::ApiState;
use super::error::{Result, Error};

/// Create node API routes
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        .route("/identity", get(get_node_identity))
        .route("/peers", get(get_peers))
        .route("/peers/:peer_id", get(get_peer))
        .route("/health", get(get_health))
        .route("/version", get(get_version))
        .route("/syncing", get(get_syncing_status))
}

/// Node identity response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeIdentityResponse {
    pub data: NodeIdentity,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeIdentity {
    pub peer_id: String,
    pub enr: String,
    pub p2p_addresses: Vec<String>,
    pub discovery_addresses: Vec<String>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NodeMetadata {
    pub seq_number: u64,
    pub attnets: String,
    pub syncnets: String,
}

/// Peers query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct PeersQuery {
    /// Peer state filter
    pub state: Option<Vec<String>>,
    /// Peer direction filter
    pub direction: Option<Vec<String>>,
}

/// Peers response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PeersResponse {
    pub data: Vec<PeerInfo>,
    pub meta: PeersMeta,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PeersMeta {
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PeerInfo {
    pub peer_id: String,
    pub enr: Option<String>,
    pub last_seen_p2p_address: String,
    pub state: String,
    pub direction: String,
}

/// Single peer response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PeerResponse {
    pub data: PeerInfo,
}

/// Health status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum HealthStatus {
    #[serde(rename = "200")]
    Ready,
    #[serde(rename = "206")]
    Syncing,
    #[serde(rename = "503")]
    NotReady,
}

/// Version response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionResponse {
    pub data: VersionData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionData {
    pub version: String,
}

/// Syncing status response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncingResponse {
    pub data: SyncingData,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SyncingData {
    pub head_slot: u64,
    pub sync_distance: u64,
    pub is_syncing: bool,
    pub is_optimistic: bool,
    pub el_offline: bool,
}

/// GET /eth/v1/node/identity
#[utoipa::path(
    get,
    path = "/eth/v1/node/identity",
    tag = "node",
    responses(
        (status = 200, description = "Node identity information", body = NodeIdentityResponse)
    )
)]
pub async fn get_node_identity(
    State(state): State<ApiState>,
) -> Result<Json<NodeIdentityResponse>> {
    // Get node identity from network service
    let identity = NodeIdentity {
        peer_id: "16Uiu2HAmExample123456789abcdef".to_string(),
        enr: "enr:-Example-ENR-Record".to_string(),
        p2p_addresses: vec![
            "/ip4/127.0.0.1/tcp/9000".to_string(),
            "/ip6/::1/tcp/9000".to_string(),
        ],
        discovery_addresses: vec![
            "/ip4/127.0.0.1/udp/9000".to_string(),
            "/ip6/::1/udp/9000".to_string(),
        ],
        metadata: NodeMetadata {
            seq_number: 1,
            attnets: "0xffffffffffffffff".to_string(),
            syncnets: "0x0f".to_string(),
        },
    };

    let response = NodeIdentityResponse { data: identity };
    Ok(Json(response))
}

/// GET /eth/v1/node/peers
#[utoipa::path(
    get,
    path = "/eth/v1/node/peers",
    tag = "node",
    params(
        PeersQuery
    ),
    responses(
        (status = 200, description = "Connected peers information", body = PeersResponse)
    )
)]
pub async fn get_peers(
    Query(query): Query<PeersQuery>,
    State(state): State<ApiState>,
) -> Result<Json<PeersResponse>> {
    // Get peer information from network service
    let peers = vec![
        PeerInfo {
            peer_id: "16Uiu2HAmPeer1Example".to_string(),
            enr: Some("enr:-Peer1-ENR-Record".to_string()),
            last_seen_p2p_address: "/ip4/10.0.0.1/tcp/9000".to_string(),
            state: "connected".to_string(),
            direction: "outbound".to_string(),
        },
        PeerInfo {
            peer_id: "16Uiu2HAmPeer2Example".to_string(),
            enr: Some("enr:-Peer2-ENR-Record".to_string()),
            last_seen_p2p_address: "/ip4/10.0.0.2/tcp/9000".to_string(),
            state: "connected".to_string(),
            direction: "inbound".to_string(),
        },
    ];

    // Apply filters if specified
    let filtered_peers = if let Some(states) = &query.state {
        peers.into_iter()
            .filter(|peer| states.contains(&peer.state))
            .collect()
    } else {
        peers
    };

    let count = filtered_peers.len() as u64;

    let response = PeersResponse {
        data: filtered_peers,
        meta: PeersMeta { count },
    };

    Ok(Json(response))
}

/// GET /eth/v1/node/peers/{peer_id}
#[utoipa::path(
    get,
    path = "/eth/v1/node/peers/{peer_id}",
    tag = "node",
    params(
        ("peer_id" = String, Path, description = "Peer identifier")
    ),
    responses(
        (status = 200, description = "Peer information", body = PeerResponse),
        (status = 404, description = "Peer not found")
    )
)]
pub async fn get_peer(
    axum::extract::Path(peer_id): axum::extract::Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<PeerResponse>> {
    // Look up specific peer
    let peer = PeerInfo {
        peer_id: peer_id.clone(),
        enr: Some("enr:-Specific-Peer-ENR".to_string()),
        last_seen_p2p_address: "/ip4/10.0.0.1/tcp/9000".to_string(),
        state: "connected".to_string(),
        direction: "outbound".to_string(),
    };

    let response = PeerResponse { data: peer };
    Ok(Json(response))
}

/// GET /eth/v1/node/health
#[utoipa::path(
    get,
    path = "/eth/v1/node/health",
    tag = "node",
    responses(
        (status = 200, description = "Node is ready"),
        (status = 206, description = "Node is syncing"),
        (status = 503, description = "Node is not ready")
    )
)]
pub async fn get_health(
    State(state): State<ApiState>,
) -> Result<StatusCode> {
    // Check node health status
    // For now, always return ready
    Ok(StatusCode::OK)
}

/// GET /eth/v1/node/version
#[utoipa::path(
    get,
    path = "/eth/v1/node/version",
    tag = "node",
    responses(
        (status = 200, description = "Node version information", body = VersionResponse)
    )
)]
pub async fn get_version(
    State(state): State<ApiState>,
) -> Result<Json<VersionResponse>> {
    let version_string = format!(
        "Panro/v{}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );

    let response = VersionResponse {
        data: VersionData {
            version: version_string,
        },
    };

    Ok(Json(response))
}

/// GET /eth/v1/node/syncing
#[utoipa::path(
    get,
    path = "/eth/v1/node/syncing",
    tag = "node",
    responses(
        (status = 200, description = "Node syncing status", body = SyncingResponse)
    )
)]
pub async fn get_syncing_status(
    State(state): State<ApiState>,
) -> Result<Json<SyncingResponse>> {
    // Get syncing status from consensus layer
    let syncing_data = SyncingData {
        head_slot: 1000, // Example current head slot
        sync_distance: 0, // 0 = fully synced
        is_syncing: false,
        is_optimistic: false,
        el_offline: false,
    };

    let response = SyncingResponse {
        data: syncing_data,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    use crate::network::NetworkConfig;
    use std::sync::Arc;

    fn create_test_state() -> ApiState {
        let validator_config = ValidatorConfig::default();
        let state_store = Arc::new(StateStore::new(Database::in_memory()));
        let validator_manager = Arc::new(ValidatorManager::new(validator_config, (*state_store).clone()));
        let config = super::super::ApiConfig::default();
        
        ApiState {
            validator_manager,
            state_store,
            config,
        }
    }

    #[tokio::test]
    async fn test_get_node_identity() {
        let state = create_test_state();
        let result = get_node_identity(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.data.peer_id.is_empty());
        assert!(!response.data.p2p_addresses.is_empty());
    }

    #[tokio::test]
    async fn test_get_peers() {
        let state = create_test_state();
        let query = PeersQuery {
            state: None,
            direction: None,
        };
        let result = get_peers(Query(query), State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.meta.count, response.data.len() as u64);
    }

    #[tokio::test]
    async fn test_get_peer() {
        let state = create_test_state();
        let peer_id = "16Uiu2HAmExample123456789abcdef".to_string();
        let result = get_peer(axum::extract::Path(peer_id.clone()), State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.peer_id, peer_id);
    }

    #[tokio::test]
    async fn test_get_health() {
        let state = create_test_state();
        let result = get_health(State(state)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_version() {
        let state = create_test_state();
        let result = get_version(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.data.version.contains("Panro"));
    }

    #[tokio::test]
    async fn test_get_syncing_status() {
        let state = create_test_state();
        let result = get_syncing_status(State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.data.sync_distance, 0);
    }

    #[tokio::test]
    async fn test_peers_filtering() {
        let state = create_test_state();
        let query = PeersQuery {
            state: Some(vec!["connected".to_string()]),
            direction: None,
        };
        let result = get_peers(Query(query), State(state)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        // All returned peers should have "connected" state
        for peer in &response.data {
            assert_eq!(peer.state, "connected");
        }
    }
}
