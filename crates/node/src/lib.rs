//! Ethean Lean Consensus Client — node library (Lean modules only).
//!
//! Replaced Beacon/Panro trees under this crate were deleted after
//! `ethean-*` crate cutovers (api, consensus, legacy network/storage, …).

#![forbid(unsafe_code)]

pub mod api_ssz;
pub mod api_view;
pub mod agg_pin;
pub mod aggregation;
pub mod aggregation_duty;
pub mod aggregation_gossip;
pub mod api_events;
pub mod block_builder;
pub mod block_payloads;
pub mod block_prune;
pub mod blocks_sync;
pub mod boot_network;
pub mod bootnodes_parse;
pub mod chain_head;
pub mod chain_fc;
pub mod chain_fc_rebuild;
pub mod chain_fc_votes;
pub mod chain_known;
pub mod chain_owner;
pub mod checkpoint_http;
pub mod checkpoint_sync;
pub mod chain_persist;
pub mod chain_redb;
pub mod chain_snap;
pub mod cli;
pub mod cli_resolve;
pub mod client;
pub mod client_boot;
pub mod client_data_dir;
pub mod client_start;
pub mod client_swarm;
pub mod clock;
pub mod commands;
pub mod crypto_status;
pub mod dispatch;
pub mod duty_attest;
pub mod duty_loop;
pub mod duty_mesh;
pub mod duty_network;
pub mod duty_propose;
pub mod duty_propose_gate;
pub mod duty_step;
pub mod events;
pub mod fork_digest_policy;
pub mod genesis_bundle;
pub mod gossip_attestation;
pub mod gossip_decode;
pub mod gossip_ingest;
pub mod gossip_stf;
pub mod lean_metrics;
pub mod local_attester;
pub mod local_finality;
pub mod local_genesis;
pub mod local_proposer;
pub mod local_status;
pub mod metrics_snap;
pub mod network;
pub mod network_target;
pub mod observability;
pub mod persist_paths;
pub mod persist_ssz;
pub mod proof_collect;
pub mod proof_service;
pub mod registry_apply;
pub mod registry_keys;
pub mod registry_keys_view;
pub mod serve_cache_seed;
pub mod shutdown;
pub mod signal_loop;
pub mod start_config;
pub mod status_handshake;
pub mod sync_catchup;
#[cfg(feature = "libp2p-quic")]
pub mod swarm_pump;
#[cfg(feature = "libp2p-quic")]
pub mod swarm_pump_agg;
pub mod sync_orphan;
pub mod test_driver;
pub mod validator_registry;
pub mod wall_loop;
pub mod wall_tick;

pub use ethean_genesis::{
    genesis_from_lean_config, load_genesis_ssz, load_lean_network_config, local_smoke_genesis,
    BuiltGenesis, ClockError, FakeTime, GenesisBuilder, GenesisError, LeanNetworkConfig, SlotClock,
    SystemTimeSource, TimeSource,
};
pub use ethean_primitives::{Epoch, Hash32, Slot, ValidatorIndex};
pub use ethean_profile::{lstar_devnet, ChainProfile, ForkId};
pub use ethean_types::{Attestation, Block, Checkpoint, SignedBlock, State, TypesError, Validator};

pub use bootnodes_parse::{parse_bootnodes_value, BootnodesSpec};
pub use chain_owner::{ChainOwner, ChainSnapshot};
pub use cli::StartArgs;
pub use cli_resolve::{
    flag_env_or, parse_subnet_ids, resolve_genesis_source, resolve_node_key, resolve_roles,
    GenesisSource, NodeKeySource, RoleFlags,
};
pub use client::EtheanClient;
pub use commands::ChainCommand;
pub use dispatch::apply_command;
pub use duty_loop::{run_duty_loop, DutyLoopConfig};
pub use ethean_network::{ListenIdentity, NodeKey};
pub use events::ChainEvent;
pub use network_target::{NetworkId, NetworkTarget};
pub use observability::{smoke_health_route, NodeObservability};
pub use registry_keys::{load_node_keys, LoadedNodeKeys};
pub use shutdown::{ShutdownPhase, ShutdownState};
pub use signal_loop::run_until_signal;
pub use start_config::{LocalRoles, MetricsListen, RpcListen, RunMode, StartConfig};
pub use validator_registry::{
    load_registry_key_rows, load_validator_assignment, parse_registry_key_rows,
    parse_validator_assignment, RegistryKeyRow, ValidatorAssignment,
};
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

#[cfg(test)]
mod api_fixture_tests;
