//! Publish window checks for completed proposal / attestation work.

use ethean_validator::DutyTick;

/// Whether a completed duty may still be published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishDecision {
    /// Within the allowed window.
    Allow,
    /// Past the deadline; discard without mutating chain state.
    TooLate,
    /// Generation mismatch; discard.
    StaleGeneration,
}

/// Decide publish eligibility given the producing tick and current tick.
pub fn decide_publish(
    produced: DutyTick,
    now: DutyTick,
    allow_same_slot_only: bool,
) -> PublishDecision {
    if produced.generation != now.generation {
        return PublishDecision::StaleGeneration;
    }
    if allow_same_slot_only && produced.slot != now.slot {
        return PublishDecision::TooLate;
    }
    if produced.slot.get() > now.slot.get() {
        return PublishDecision::TooLate;
    }
    PublishDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;

    #[test]
    fn rejects_stale_generation() {
        let a = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 1,
        };
        let b = DutyTick {
            slot: Slot::new(1),
            interval: 0,
            generation: 2,
        };
        assert_eq!(decide_publish(a, b, true), PublishDecision::StaleGeneration);
    }
}
