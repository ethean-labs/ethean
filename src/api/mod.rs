//! REST API Server Module for Panro Beacon Chain Client

pub mod beacon;
pub mod validator;
pub mod node;
pub mod config;
pub mod debug;
pub mod middleware;
pub mod types;
pub mod error;
pub mod websocket;

pub use types::*;

use axum::{Router, routing::get, response::Json};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::consensus::validator_management::ValidatorManager;
use crate::storage::StateStore;

/// API server state
#[derive(Clone)]
pub struct ApiState {
    pub validator_manager: Arc<ValidatorManager>,
    pub state_store: Arc<StateStore>,
    pub config: ApiConfig,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub bind_addr: std::net::SocketAddr,
    pub max_request_size: usize,
    pub enable_cors: bool,
    pub enable_compression: bool,
    pub rate_limit: RateLimitConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:5052".parse().unwrap(),
            max_request_size: 1024 * 1024,
            enable_cors: true,
            enable_compression: true,
            rate_limit: RateLimitConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 120,
            burst_size: 10,
            window_size_seconds: 60,
        }
    }
}

/// API server
pub struct ApiServer {
    router: Router,
    state: ApiState,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(
        validator_manager: Arc<ValidatorManager>,
        state_store: Arc<StateStore>,
        config: ApiConfig,
    ) -> Self {
        let state = ApiState {
            validator_manager,
            state_store,
            config,
        };

        let router = Self::create_router(state.clone());
        Self { router, state }
    }

    /// Create the main router
    fn create_router(state: ApiState) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .nest("/eth/v1", Self::create_v1_routes())
            .with_state(state)
    }

    /// Create API v1 routes
    fn create_v1_routes() -> Router<ApiState> {
        Router::new()
            .nest("/beacon", beacon::create_routes())
            .nest("/validator", validator::create_routes())
            .nest("/node", node::create_routes())
            .nest("/config", config::create_routes())
            .nest("/debug", debug::create_routes())
            .nest("/ws", websocket::create_routes())
    }

    /// Start the API server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind(self.state.config.bind_addr).await?;
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "panro-beacon-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
