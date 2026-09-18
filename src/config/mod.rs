//! Configuration module for Ethean

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API server configuration
    pub api: ApiConfig,
    /// Validator configuration
    pub validator: ValidatorConfig,
    /// Network configuration
    pub network: NetworkConfig,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Bind address for API server
    pub bind_addr: SocketAddr,
    /// Enable CORS
    pub enable_cors: bool,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Enable validator mode
    pub enabled: bool,
    /// Validator keys directory
    pub keys_dir: String,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_addr: SocketAddr,
    /// Boot nodes
    pub boot_nodes: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            validator: ValidatorConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:5052".parse().unwrap(),
            enable_cors: true,
        }
    }
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            keys_dir: "./validator_keys".to_string(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:9000".parse().unwrap(),
            boot_nodes: vec![],
        }
    }
}

/// Config result type
pub type Result<T> = std::result::Result<T, Error>;

/// Config errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid config")]
    Invalid,
    
    #[error("File not found")]
    FileNotFound,
}
