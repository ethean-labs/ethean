//! Injectable Lean slot clock.
//!
//! `GenesisConfig.genesis_time` is Unix **seconds** (leanSpec `Uint64`).
//! Convert to milliseconds with `checked_mul(1000)` before slot math.

use std::time::{SystemTime, UNIX_EPOCH};

use ethean_primitives::Slot;
use ethean_profile::ChainProfile;

use crate::error::{ClockError, GenesisError};

/// Source of Unix time in milliseconds.
pub trait TimeSource {
    fn unix_millis(&self) -> Result<u64, ClockError>;
}

/// Settable clock for tests. Optionally rejects regressions.
#[derive(Debug, Clone)]
pub struct FakeTime {
    millis: u64,
    reject_regress: bool,
}

impl FakeTime {
    pub fn new(millis: u64) -> Self {
        Self {
            millis,
            reject_regress: false,
        }
    }

    pub fn rejecting_regress(millis: u64) -> Self {
        Self {
            millis,
            reject_regress: true,
        }
    }

    pub fn set_millis(&mut self, millis: u64) -> Result<(), ClockError> {
        if self.reject_regress && millis < self.millis {
            return Err(ClockError::BackwardTime);
        }
        self.millis = millis;
        Ok(())
    }

    pub fn millis(&self) -> u64 {
        self.millis
    }
}

impl TimeSource for FakeTime {
    fn unix_millis(&self) -> Result<u64, ClockError> {
        Ok(self.millis)
    }
}

/// Wall clock for node operations only — not for pure consensus crates.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn unix_millis(&self) -> Result<u64, ClockError> {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ClockError::SystemClockUnavailable)?;
        u64::try_from(dur.as_millis()).map_err(|_| ClockError::TimestampOverflow)
    }
}

/// Maps Unix millis to Lean slots using a pinned [`ChainProfile`].
#[derive(Debug, Clone)]
pub struct SlotClock {
    genesis_time_secs: u64,
    profile: ChainProfile,
}

impl SlotClock {
    pub fn new(genesis_time_secs: u64, profile: ChainProfile) -> Result<Self, GenesisError> {
        profile
            .validate()
            .map_err(|e| GenesisError::Profile(e.to_string()))?;
        if profile.milliseconds_per_slot == 0 || profile.milliseconds_per_interval == 0 {
            return Err(GenesisError::Clock(ClockError::ZeroDuration));
        }
        Ok(Self {
            genesis_time_secs,
            profile,
        })
    }

    pub fn profile(&self) -> &ChainProfile {
        &self.profile
    }

    pub fn genesis_time_secs(&self) -> u64 {
        self.genesis_time_secs
    }

    /// Genesis instant in Unix milliseconds.
    pub fn genesis_time_millis(&self) -> Result<u64, ClockError> {
        self.genesis_time_secs
            .checked_mul(1000)
            .ok_or(ClockError::TimestampOverflow)
    }

    /// Slot containing `now_ms`. Errors if before genesis.
    pub fn slot_at_millis(&self, now_ms: u64) -> Result<Slot, ClockError> {
        let genesis_ms = self.genesis_time_millis()?;
        if now_ms < genesis_ms {
            return Err(ClockError::PreGenesis);
        }
        let elapsed = now_ms - genesis_ms;
        let slot = elapsed
            .checked_div(self.profile.milliseconds_per_slot)
            .ok_or(ClockError::ZeroDuration)?;
        Ok(Slot::new(slot))
    }

    /// `(slot, interval_index)` where interval is in `0..intervals_per_slot`.
    pub fn interval_at_millis(&self, now_ms: u64) -> Result<(Slot, u64), ClockError> {
        let genesis_ms = self.genesis_time_millis()?;
        if now_ms < genesis_ms {
            return Err(ClockError::PreGenesis);
        }
        let elapsed = now_ms - genesis_ms;
        let ms_slot = self.profile.milliseconds_per_slot;
        let ms_iv = self.profile.milliseconds_per_interval;
        let slot = elapsed.checked_div(ms_slot).ok_or(ClockError::ZeroDuration)?;
        let within = elapsed % ms_slot;
        let interval = within.checked_div(ms_iv).ok_or(ClockError::ZeroDuration)?;
        Ok((Slot::new(slot), interval))
    }

    /// Start time of `slot` in Unix milliseconds.
    pub fn slot_start_millis(&self, slot: Slot) -> Result<u64, ClockError> {
        let genesis_ms = self.genesis_time_millis()?;
        let offset = slot
            .get()
            .checked_mul(self.profile.milliseconds_per_slot)
            .ok_or(ClockError::TimestampOverflow)?;
        genesis_ms
            .checked_add(offset)
            .ok_or(ClockError::TimestampOverflow)
    }

    /// Current slot from a [`TimeSource`].
    pub fn slot_now<T: TimeSource>(&self, time: &T) -> Result<Slot, ClockError> {
        self.slot_at_millis(time.unix_millis()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_profile::lstar_devnet;

    fn clock(genesis_secs: u64) -> SlotClock {
        SlotClock::new(genesis_secs, lstar_devnet().unwrap()).unwrap()
    }

    #[test]
    fn exact_four_second_boundaries() {
        let c = clock(1_700_000_000);
        let g = c.genesis_time_millis().unwrap();
        assert_eq!(c.slot_at_millis(g).unwrap(), Slot::ZERO);
        assert_eq!(c.slot_at_millis(g + 3999).unwrap(), Slot::ZERO);
        assert_eq!(c.slot_at_millis(g + 4000).unwrap(), Slot::new(1));
        assert_eq!(c.slot_start_millis(Slot::new(2)).unwrap(), g + 8000);
        let (s, iv) = c.interval_at_millis(g + 800).unwrap();
        assert_eq!(s, Slot::ZERO);
        assert_eq!(iv, 1);
    }

    #[test]
    fn pre_genesis() {
        let c = clock(100);
        assert_eq!(
            c.slot_at_millis(99_999),
            Err(ClockError::PreGenesis)
        );
    }

    #[test]
    fn genesis_secs_overflow_to_millis() {
        let c = clock(u64::MAX / 1000 + 1);
        assert_eq!(c.genesis_time_millis(), Err(ClockError::TimestampOverflow));
    }

    #[test]
    fn fake_time_rejects_regress() {
        let mut fake = FakeTime::rejecting_regress(5_000);
        assert!(fake.set_millis(6_000).is_ok());
        assert_eq!(fake.set_millis(5_000), Err(ClockError::BackwardTime));
    }

    #[test]
    fn slot_now_uses_time_source() {
        let c = clock(1_000);
        let fake = FakeTime::new(1_000_000 + 8_000);
        assert_eq!(c.slot_now(&fake).unwrap(), Slot::new(2));
    }
}
