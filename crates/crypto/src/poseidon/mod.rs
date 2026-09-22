//! Poseidon1 (Hades) permutation over KoalaBear, widths 16 and 24, `x^3` S-box.
//!
//! Round structure, verbatim from leanSpec `poseidon.py`:
//! 4 full rounds, then `R_P` partial rounds (S-box on `state[0]` only), then
//! 4 full rounds. Each round adds a full width of constants and applies the
//! circulant MDS matrix. There is no initial linear layer.

mod mds;
mod modes;
mod rc16;
mod rc24;

pub use mds::{MDS_ROW_16, MDS_ROW_24};
pub use modes::{compress, safe_domain_separator, sponge, Poseidon24Input};

use crate::field::Fp;

/// Number of full rounds (split evenly around the partial rounds).
pub const FULL_ROUNDS: usize = 8;
/// Partial rounds for width 16.
pub const PARTIAL_ROUNDS_16: usize = 20;
/// Partial rounds for width 24.
pub const PARTIAL_ROUNDS_24: usize = 23;

#[inline(always)]
fn add_constants<const W: usize>(state: &mut [Fp; W], rc: &[u32; W]) {
    for (s, &c) in state.iter_mut().zip(rc.iter()) {
        *s = s.add(Fp::new_const(c));
    }
}

#[inline]
fn permute<const W: usize>(state: &mut [Fp; W], constants: &[[u32; W]], mds_row: &[u32; W]) {
    let half = FULL_ROUNDS / 2;
    let partial = constants.len() - FULL_ROUNDS;
    let (initial, rest) = constants.split_at(half);
    let (middle, terminal) = rest.split_at(partial);
    for rc in initial {
        add_constants(state, rc);
        for s in state.iter_mut() {
            *s = s.cube();
        }
        mds::circulant_multiply(state, mds_row);
    }
    for rc in middle {
        add_constants(state, rc);
        state[0] = state[0].cube();
        mds::circulant_multiply(state, mds_row);
    }
    for rc in terminal {
        add_constants(state, rc);
        for s in state.iter_mut() {
            *s = s.cube();
        }
        mds::circulant_multiply(state, mds_row);
    }
}

/// Width-16 permutation, in place.
pub fn permute16(state: &mut [Fp; 16]) {
    permute(state, &rc16::RC_16, &MDS_ROW_16);
}

/// Width-24 permutation, in place.
pub fn permute24(state: &mut [Fp; 24]) {
    permute(state, &rc24::RC_24, &MDS_ROW_24);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe<const N: usize>(values: [u32; N]) -> [Fp; N] {
        values.map(|v| Fp::new(v).unwrap())
    }

    #[test]
    fn constant_table_shapes() {
        assert_eq!(rc16::RC_16.len(), FULL_ROUNDS + PARTIAL_ROUNDS_16);
        assert_eq!(rc24::RC_24.len(), FULL_ROUNDS + PARTIAL_ROUNDS_24);
        assert_eq!(rc16::RC_16[0][0], 0x7EE56A48);
        assert_eq!(rc24::RC_24[0][0], 0x1D0939DC);
    }

    /// leanSpec `tests/spec/crypto/test_poseidon.py`, width 16, input `0..16`.
    #[test]
    fn width16_known_answer() {
        let mut state: [Fp; 16] = std::array::from_fn(|i| Fp::new(i as u32).unwrap());
        permute16(&mut state);
        let expected = fe([
            610090613, 935319874, 1893335292, 796792199, 356405232, 552237741, 55134556,
            1215104204, 1823723405, 1133298033, 1780633798, 1453946561, 710069176, 1128629550,
            1917333254, 1175481618,
        ]);
        assert_eq!(state, expected);
    }

    /// leanSpec `tests/spec/crypto/test_poseidon.py`, width 24, input `0..24`.
    #[test]
    fn width24_known_answer() {
        let mut state: [Fp; 24] = std::array::from_fn(|i| Fp::new(i as u32).unwrap());
        permute24(&mut state);
        let expected = fe([
            511672087, 215882318, 237782537, 740528428, 712760904, 54615367, 751514671, 110231969,
            1905276435, 992525666, 918312360, 18628693, 749929200, 1916418953, 691276896,
            1112901727, 1163558623, 882867603, 673396520, 1480278156, 1402044758, 1693467175,
            1766273044, 433841551,
        ]);
        assert_eq!(state, expected);
    }
}
