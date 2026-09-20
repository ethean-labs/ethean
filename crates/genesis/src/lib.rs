//! Deterministic Lean genesis and injectable slot clock.

#![forbid(unsafe_code)]

mod builder;
mod clock;
mod config_yaml;
mod error;
mod hex;
mod loader;

pub use builder::{local_smoke_genesis, BuiltGenesis, GenesisBuilder};
pub use clock::{FakeTime, SlotClock, SystemTimeSource, TimeSource};
pub use config_yaml::{
    genesis_from_lean_config, load_lean_network_config, parse_lean_network_config, LeanNetworkConfig,
};
pub use error::{ClockError, GenesisError};
pub use loader::load_genesis_ssz;
