//! Slot/interval scheduler events (profile-driven, not hard-coded sleeps).

use ethean_primitives::Slot;
use ethean_profile::ChainProfile;

/// Monotonic duty tick: slot, interval within the slot, and generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DutyTick {
    /// Absolute slot.
    pub slot: Slot,
    /// Interval index in `[0, intervals_per_slot)`.
    pub interval: u8,
    /// Generation incremented on clock catch-up / restart (stale rejection).
    pub generation: u64,
}

/// Convert elapsed milliseconds since genesis into a duty tick.
pub fn tick_from_elapsed_ms(elapsed_ms: u64, profile: &ChainProfile, generation: u64) -> DutyTick {
    let slot_ms = profile.milliseconds_per_slot.max(1);
    let intervals = profile.intervals_per_slot.max(1) as u64;
    let interval_ms = (slot_ms / intervals).max(1);
    let slot = elapsed_ms / slot_ms;
    let rem = elapsed_ms % slot_ms;
    let interval = (rem / interval_ms).min(intervals - 1) as u8;
    DutyTick {
        slot: Slot::new(slot),
        interval,
        generation,
    }
}

/// Advance one interval; wraps to the next slot when needed.
pub fn advance_tick(tick: DutyTick, profile: &ChainProfile) -> DutyTick {
    let max_i = profile.intervals_per_slot.saturating_sub(1);
    if tick.interval < max_i {
        DutyTick {
            slot: tick.slot,
            interval: tick.interval + 1,
            generation: tick.generation,
        }
    } else {
        DutyTick {
            slot: Slot::new(tick.slot.get().saturating_add(1)),
            interval: 0,
            generation: tick.generation,
        }
    }
}

/// Deduplicate identical ticks; returns true when `next` should be processed.
pub fn should_process(last: Option<DutyTick>, next: DutyTick) -> bool {
    match last {
        None => true,
        Some(prev) => next != prev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_profile::lstar_devnet;

    #[test]
    fn five_intervals_per_four_second_slot() {
        let p = lstar_devnet();
        assert_eq!(p.milliseconds_per_slot, 4000);
        assert_eq!(p.intervals_per_slot, 5);
        let t0 = tick_from_elapsed_ms(0, &p, 1);
        assert_eq!(t0.slot.get(), 0);
        assert_eq!(t0.interval, 0);
        let t_mid = tick_from_elapsed_ms(1600, &p, 1);
        assert_eq!(t_mid.interval, 2);
        let t_next = tick_from_elapsed_ms(4000, &p, 1);
        assert_eq!(t_next.slot.get(), 1);
        assert_eq!(t_next.interval, 0);
    }

    #[test]
    fn dedupes_repeats() {
        let t = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        assert!(!should_process(Some(t), t));
        assert!(should_process(Some(t), advance_tick(t, &lstar_devnet())));
    }
}
