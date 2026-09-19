//! Node-facing slot clock wrapping `ethean-genesis`.

pub use ethean_genesis::{
    ClockError, FakeTime, GenesisError, SlotClock, SystemTimeSource, TimeSource,
};

use ethean_profile::ChainProfile;
use ethean_types::State;

/// Build a [`SlotClock`] from genesis state + pinned profile.
pub fn clock_from_genesis(
    genesis: &State,
    profile: ChainProfile,
) -> Result<SlotClock, GenesisError> {
    SlotClock::new(genesis.genesis_time(), profile)
}

/// Wall-clock source for node scheduling (not for pure consensus tests).
pub fn system_time_source() -> SystemTimeSource {
    SystemTimeSource
}
