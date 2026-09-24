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

/// Integer square root (floor), matching Python `math.isqrt`.
fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = (n as f64).sqrt() as u64;
    while x.checked_mul(x).is_none_or(|sq| sq > n) {
        x -= 1;
    }
    while (x + 1).checked_mul(x + 1).is_some_and(|sq| sq <= n) {
        x += 1;
    }
    x
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
    fn isqrt_matches_reference() {
        for n in 0u64..10_000 {
            let mut r = 0u64;
            while (r + 1) * (r + 1) <= n {
                r += 1;
            }
            assert_eq!(isqrt(n), r, "n={n}");
        }
        for n in [u64::MAX, u64::MAX - 1, (1u64 << 63) + 7, 999_999_999_999_999_999] {
            let r = isqrt(n);
            assert!(r.checked_mul(r).is_some_and(|sq| sq <= n));
            assert!((r + 1).checked_mul(r + 1).is_none_or(|sq| sq > n));
        }
    }

    #[test]
    fn square_and_pronic() {
        let f = Slot::ZERO;
        assert!(is_justifiable_after(Slot::new(9), f));
        assert!(is_justifiable_after(Slot::new(6), f));
        assert!(!is_justifiable_after(Slot::new(7), f));
    }
}
