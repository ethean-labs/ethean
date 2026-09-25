//! Deterministic Lean genesis and injectable slot clock.

#![forbid(unsafe_code)]

mod builder;
mod clock;
mod config_yaml;
mod error;
mod hex;
mod leanspec_pins;
mod loader;

pub use builder::{
    local_smoke_genesis, BuiltGenesis, GenesisBuilder, EMPTY_BLOCK_BODY_ROOT,
};
pub use clock::{FakeTime, SlotClock, SystemTimeSource, TimeSource};
pub use config_yaml::{
    genesis_from_lean_config, load_lean_network_config, parse_lean_network_config, LeanNetworkConfig,
};
pub use error::{ClockError, GenesisError};
pub use hex::decode_hex_fixed;
pub use leanspec_pins::{
    prod_scheme_genesis, seal_genesis_header, PROD1_GENESIS_BLOCK_ROOT, PROD4_GENESIS_BLOCK_ROOT,
    PROD4_GENESIS_STATE_ROOT,
};
pub use loader::load_genesis_ssz;
