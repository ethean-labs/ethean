//! Slot advancement helpers (3SF-mini justifiable distances).

use ethean_primitives::Slot;

/// First N slots after finalization are always justifiable (leanSpec).
pub const IMMEDIATE_JUSTIFICATION_WINDOW: u64 = 5;

/// Relative bitfield index for `target` after `finalized`, or `None` if at/before finalized.
pub fn justified_index_after(target: Slot, finalized: Slot) -> Option<usize> {
    if target.get() <= finalized.get() {
        return None;
    }
    Some((target.get() - finalized.get() - 1) as usize)
}

/// Whether `target` is a valid justification candidate after `finalized` (3SF-mini).
pub fn is_justifiable_after(target: Slot, finalized: Slot) -> bool {
    if target.get() < finalized.get() {
        return false;
    }
    let delta = target.get() - finalized.get();
    if delta <= IMMEDIATE_JUSTIFICATION_WINDOW {
        return true;
    }
    let root = isqrt(delta);
    if root * root == delta {
        return true;
    }
    let disc = 4 * delta + 1;
    let disc_root = isqrt(disc);
    disc_root * disc_root == disc && disc_root % 2 == 1
}

fn isqrt(n: u64) -> u64 {
    (n as f64).sqrt() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immediate_window() {
        let f = Slot::new(10);
        assert!(is_justifiable_after(Slot::new(10), f));
        assert!(is_justifiable_after(Slot::new(15), f));
        // delta=6 is pronic; delta=7 is neither window/square/pronic
        assert!(is_justifiable_after(Slot::new(16), f));
        assert!(!is_justifiable_after(Slot::new(17), f));
    }

    #[test]
    fn square_and_pronic() {
        let f = Slot::ZERO;
        assert!(is_justifiable_after(Slot::new(9), f));
        assert!(is_justifiable_after(Slot::new(6), f));
        assert!(!is_justifiable_after(Slot::new(7), f));
    }
}
