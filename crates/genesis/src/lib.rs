//! Deterministic Lean genesis and injectable slot clock.

#![forbid(unsafe_code)]

mod builder;
mod clock;
mod error;
mod loader;

pub use builder::{local_smoke_genesis, BuiltGenesis, GenesisBuilder};
pub use clock::{FakeTime, SlotClock, SystemTimeSource, TimeSource};
pub use error::{ClockError, GenesisError};
pub use loader::load_genesis_ssz;
