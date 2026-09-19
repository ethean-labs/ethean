//! Ethean Lean Consensus Client — node library (Lean modules only).
//!
//! Replaced Beacon/Panro trees under this crate were deleted after
//! `ethean-*` crate cutovers (api, consensus, legacy network/storage, …).

#![forbid(unsafe_code)]

pub mod aggregation;
pub mod block_builder;
pub mod boot_network;
pub mod chain_owner;
pub mod cli;
pub mod client;
#[cfg(feature = "libp2p-quic")]
pub mod client_swarm;
pub mod clock;
pub mod commands;
pub mod crypto;
pub mod dispatch;
pub mod duty_loop;
pub mod duty_step;
pub mod events;
pub mod gossip_decode;
pub mod gossip_ingest;
pub mod gossip_pool;
pub mod gossip_stf;
pub mod network;
pub mod observability;
pub mod shutdown;
pub mod signal_loop;
pub mod start_config;
#[cfg(feature = "libp2p-quic")]
pub mod swarm_pump;
pub mod wall_loop;
pub mod wall_tick;

pub use ethean_primitives::{Epoch, Hash32, Slot, ValidatorIndex};
pub use ethean_profile::{lstar_devnet, ChainProfile, ForkId};
pub use ethean_types::{
    Attestation, Block, Checkpoint, SignedBlock, State, TypesError, Validator,
};
pub use ethean_genesis::{
    load_genesis_ssz, local_smoke_genesis, BuiltGenesis, ClockError, FakeTime, GenesisBuilder,
    GenesisError, SlotClock, SystemTimeSource, TimeSource,
};

pub use chain_owner::{ChainOwner, ChainSnapshot};
pub use client::EtheanClient;
pub use commands::ChainCommand;
pub use dispatch::apply_command;
pub use duty_loop::{run_duty_loop, DutyLoopConfig};
pub use events::ChainEvent;
pub use observability::{smoke_health_route, NodeObservability};
pub use shutdown::{ShutdownPhase, ShutdownState};
pub use signal_loop::run_until_signal;
pub use start_config::{RunMode, StartConfig};
pub use wall_loop::{run_wall_duty_loop, WallLoopConfig};
pub use wall_tick::{ms_until_next_interval, tick_from_wall};

/// Main result type for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the application
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Types error: {0}")]
    Types(#[from] TypesError),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(#[from] ethean_network::NetworkError),

    #[error("Storage error: {0}")]
    Storage(#[from] ethean_storage::StorageError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Profile error: {0}")]
    Profile(#[from] ethean_profile::ProfileError),

    #[error("Genesis error: {0}")]
    Genesis(#[from] ethean_genesis::GenesisError),

    #[error("Sync error: {0}")]
    Sync(#[from] ethean_sync::SyncError),

    #[error("Metrics error: {0}")]
    Metrics(#[from] ethean_metrics::MetricsError),

    #[error("RPC error: {0}")]
    Rpc(#[from] ethean_rpc::RpcError),

    #[error("Clock error: {0}")]
    Clock(#[from] ethean_genesis::ClockError),
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Crate package name
pub const NAME: &str = env!("CARGO_PKG_NAME");
