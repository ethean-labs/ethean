//! Wall-clock duty tick helpers (no sleep; caller schedules waits).

use ethean_genesis::{ClockError, SlotClock, TimeSource};
use ethean_validator::{tick_from_elapsed_ms, DutyTick};

/// Build a [`DutyTick`] from an injectable clock and time source.
pub fn tick_from_wall<T: TimeSource>(
    clock: &SlotClock,
    time: &T,
    generation: u64,
) -> Result<DutyTick, ClockError> {
    let now_ms = time.unix_millis()?;
    let genesis_ms = clock.genesis_time_millis()?;
    if now_ms < genesis_ms {
        return Err(ClockError::PreGenesis);
    }
    let elapsed = now_ms - genesis_ms;
    Ok(tick_from_elapsed_ms(elapsed, clock.profile(), generation))
}

/// Milliseconds until the next profile interval boundary after `now_ms`.
pub fn ms_until_next_interval(clock: &SlotClock, now_ms: u64) -> Result<u64, ClockError> {
    let genesis_ms = clock.genesis_time_millis()?;
    if now_ms < genesis_ms {
        return Err(ClockError::PreGenesis);
    }
    let interval_ms = clock.profile().milliseconds_per_interval.max(1);
    let elapsed = now_ms - genesis_ms;
    let rem = elapsed % interval_ms;
    if rem == 0 {
        Ok(interval_ms)
    } else {
        Ok(interval_ms - rem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_genesis::FakeTime;
    use ethean_profile::lstar_devnet;

    #[test]
    fn wall_tick_matches_elapsed() {
        let profile = lstar_devnet().unwrap();
        let genesis_secs = 1_700_000_000u64;
        let clock = SlotClock::new(genesis_secs, profile).unwrap();
        let genesis_ms = clock.genesis_time_millis().unwrap();
        let time = FakeTime::new(genesis_ms + 1600);
        let tick = tick_from_wall(&clock, &time, 1).unwrap();
        assert_eq!(tick.slot.get(), 0);
        assert_eq!(tick.interval, 2);
    }

    #[test]
    fn next_interval_delta() {
        let profile = lstar_devnet().unwrap();
        let clock = SlotClock::new(1_000, profile).unwrap();
        let g = clock.genesis_time_millis().unwrap();
        assert_eq!(ms_until_next_interval(&clock, g).unwrap(), 800);
        assert_eq!(ms_until_next_interval(&clock, g + 1).unwrap(), 799);
    }
}
