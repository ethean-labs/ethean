//! REST API Server Module for Panro Beacon Chain Client
//!
//! Implements the Ethereum Beacon API specification with additional
//! developer-friendly endpoints for enhanced user experience.

pub mod beacon;
pub mod validator;
pub mod node;
pub mod config;
pub mod debug;
pub mod middleware;
pub mod types;
pub mod error;

pub use error::{Error, Result};
pub use types::*;

use axum::{
    Router,
    routing::get,
    response::Json,
    http::StatusCode,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::net::SocketAddr;

use crate::consensus::ValidatorManager;
use crate::storage::StateStore;
use crate::network::NetworkService;

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server bind address
    pub bind_addr: SocketAddr,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable compression
    pub enable_compression: bool,
    /// API rate limiting
    pub rate_limit: Option<RateLimitConfig>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:5052".parse().unwrap(),
            max_request_size: 1024 * 1024, // 1MB
            timeout_seconds: 30,
            enable_cors: true,
            enable_compression: true,
            rate_limit: Some(RateLimitConfig::default()),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute per IP
    pub requests_per_minute: u32,
    /// Burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 120,
            burst_size: 10,
        }
    }
}

/// API server state shared across handlers
#[derive(Debug, Clone)]
pub struct ApiState {
    /// Validator manager for consensus operations
    pub validator_manager: Arc<ValidatorManager>,
    /// State store for blockchain data
    pub state_store: Arc<StateStore>,
    /// Network service for peer information
    pub network_service: Arc<NetworkService>,
    /// API configuration
    pub config: ApiConfig,
}

/// API server implementation
pub struct ApiServer {
    /// Shared application state
    state: ApiState,
    /// Axum router
    router: Router,
}

impl ApiServer {
    /// Create new API server
    pub fn new(
        validator_manager: Arc<ValidatorManager>,
        state_store: Arc<StateStore>,
        network_service: Arc<NetworkService>,
        config: ApiConfig,
    ) -> Self {
        let state = ApiState {
            validator_manager,
            state_store,
            network_service,
            config: config.clone(),
        };

        let router = Self::create_router(state.clone());

        Self { state, router }
    }

    /// Create the main router with all API routes
    fn create_router(state: ApiState) -> Router {
        let middleware_stack = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive());

        Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            
            // API v1 routes
            .nest("/eth/v1", Self::create_v1_routes())
            .nest("/eth/v2", Self::create_v2_routes())
            
            // OpenAPI documentation
            .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
            
            // Apply middleware
            .layer(middleware_stack)
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
    }

    /// Create API v2 routes (future extensions)
    fn create_v2_routes() -> Router<ApiState> {
        Router::new()
            // Placeholder for future API v2 endpoints
            .route("/version", get(api_v2_version))
    }

    /// Start the API server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting API server on {}", self.state.config.bind_addr);

        let listener = tokio::net::TcpListener::bind(self.state.config.bind_addr).await?;
        
        axum::serve(listener, self.router)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Get the router for testing
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "panro-beacon-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// API v2 version endpoint
async fn api_v2_version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": "2.0",
        "status": "development",
        "features": ["enhanced_filtering", "batch_operations"]
    }))
}

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health and version endpoints will be added here
    ),
    components(
        schemas(
            // API types will be added here
        )
    ),
    tags(
        (name = "beacon", description = "Beacon chain state endpoints"),
        (name = "validator", description = "Validator management endpoints"),
        (name = "node", description = "Node information endpoints"),
        (name = "config", description = "Configuration endpoints"),
        (name = "debug", description = "Debug and diagnostic endpoints"),
    ),
    info(
        title = "Panro Beacon API",
        version = "1.0.0",
        description = "Ethereum Beacon Chain API implementation for Panro client",
        contact(
            name = "Panro Team",
            email = "team@panro.dev"
        )
    ),
    servers(
        (url = "http://localhost:5052", description = "Local development server"),
        (url = "https://api.panro.dev", description = "Production server")
    )
)]
struct ApiDoc;

/// API result type
pub type Result<T> = std::result::Result<T, Error>;

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error")]
    Http,
    
    #[error("Invalid request")]
    InvalidRequest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validator_management::{ValidatorConfig, ValidatorManager};
    use crate::storage::{StateStore, Database};
    use crate::network::{NetworkService, NetworkConfig};

    fn create_test_api_server() -> ApiServer {
        let validator_config = ValidatorConfig::default();
        let state_store = Arc::new(StateStore::new(Database::in_memory()));
        let validator_manager = Arc::new(ValidatorManager::new(validator_config, state_store.clone()));
        
        let network_config = NetworkConfig::local();
        let network_service = Arc::new(NetworkService::new(network_config).unwrap());
        
        let api_config = ApiConfig::default();
        
        ApiServer::new(validator_manager, state_store, network_service, api_config)
    }

    #[test]
    fn test_api_server_creation() {
        let _server = create_test_api_server();
        // Test passes if no panic during creation
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.bind_addr.port(), 5052);
        assert_eq!(config.max_request_size, 1024 * 1024);
        assert!(config.enable_cors);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 120);
        assert_eq!(config.burst_size, 10);
    }
}
