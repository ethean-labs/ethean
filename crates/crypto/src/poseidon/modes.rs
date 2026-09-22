//! Hashing modes built on the Poseidon1 permutation: compression (truncated
//! feed-forward), the leaf-hash domain separator, and the replacement sponge.

use super::{permute16, permute24};
use crate::field::{base_p_limbs_from_u128, Fp};

/// Sponge width used for leaves and message hashing.
pub const SPONGE_WIDTH: usize = 24;

/// Which permutation backs a compression call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Poseidon24Input {
    Width16,
    Width24,
}

/// `Truncate(Permute(pad(input)) + pad(input))` to `OUT` elements.
///
/// Input is zero-padded to the permutation width; it must not be longer than
/// the width and must contain at least `OUT` elements.
pub fn compress<const OUT: usize>(width: Poseidon24Input, input: &[Fp]) -> [Fp; OUT] {
    assert!(
        input.len() >= OUT,
        "Poseidon compress: input shorter than output"
    );
    match width {
        Poseidon24Input::Width16 => compress_with::<16, OUT>(input, permute16),
        Poseidon24Input::Width24 => compress_with::<24, OUT>(input, permute24),
    }
}

fn compress_with<const W: usize, const OUT: usize>(
    input: &[Fp],
    permute: fn(&mut [Fp; W]),
) -> [Fp; OUT] {
    assert!(
        input.len() <= W,
        "Poseidon compress: input longer than width"
    );
    let mut padded = [Fp::ZERO; W];
    padded[..input.len()].copy_from_slice(input);
    let mut state = padded;
    permute(&mut state);
    let mut out = [Fp::ZERO; OUT];
    for (o, (s, p)) in out.iter_mut().zip(state.iter().zip(padded.iter())) {
        *o = s.add(*p);
    }
    out
}

/// Capacity value for the leaf sponge: pack four `u32` lengths big-endian
/// into one integer, write it as 24 base-`p` limbs, and compress (width 24).
pub fn safe_domain_separator<const CAP: usize>(lengths: &[u32; 4]) -> [Fp; CAP] {
    let mut acc: u128 = 0;
    for &len in lengths {
        acc = (acc << 32) | len as u128;
    }
    let limbs =
        base_p_limbs_from_u128::<SPONGE_WIDTH>(acc).expect("128-bit value fits in 24 limbs");
    compress::<CAP>(Poseidon24Input::Width24, &limbs)
}

/// Replacement sponge over the width-24 permutation, capacity first.
///
/// Rate elements are overwritten (not added) by each input chunk. A trailing
/// partial chunk is zero-padded. Squeezing returns the first `OUT` rate
/// elements, permuting again only if more than one rate block is needed.
pub fn sponge<const OUT: usize>(capacity: &[Fp], input: &[Fp]) -> [Fp; OUT] {
    let cap = capacity.len();
    assert!(
        cap < SPONGE_WIDTH,
        "Poseidon sponge: capacity must leave a rate"
    );
    let rate = SPONGE_WIDTH - cap;
    let mut state = [Fp::ZERO; SPONGE_WIDTH];
    state[..cap].copy_from_slice(capacity);

    let mut chunks = input.chunks_exact(rate);
    for chunk in &mut chunks {
        state[cap..].copy_from_slice(chunk);
        permute24(&mut state);
    }
    let rem = chunks.remainder();
    if !rem.is_empty() {
        state[cap..cap + rem.len()].copy_from_slice(rem);
        state[cap + rem.len()..].fill(Fp::ZERO);
        permute24(&mut state);
    }

    let mut out = [Fp::ZERO; OUT];
    let mut written = 0;
    while written < OUT {
        let take = (OUT - written).min(rate);
        out[written..written + take].copy_from_slice(&state[cap..cap + take]);
        written += take;
        if written < OUT {
            permute24(&mut state);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(n: u32) -> Vec<Fp> {
        (0..n).map(|i| Fp::new(i).unwrap()).collect()
    }

    fn fe<const N: usize>(values: [u32; N]) -> [Fp; N] {
        values.map(|v| Fp::new(v).unwrap())
    }

    /// leanSpec `tests/spec/crypto/xmss/test_poseidon.py` vectors.
    #[test]
    fn compress_known_answers() {
        let out16 = compress::<8>(Poseidon24Input::Width16, &seq(8));
        assert_eq!(
            out16,
            fe([
                1322417907, 1303287496, 1541273089, 1618220094, 1711479283, 239494928, 1755981565,
                1393953151
            ])
        );
        let out24 = compress::<8>(Poseidon24Input::Width24, &seq(8));
        assert_eq!(
            out24,
            fe([
                1114630880, 1895839298, 1019726674, 919764788, 323823531, 372774729, 1191983079,
                70660318
            ])
        );
    }

    #[test]
    fn domain_separator_known_answer() {
        let sep = safe_domain_separator::<9>(&[5, 2, 4, 8]);
        assert_eq!(
            sep,
            fe([
                627826400, 1244476188, 370678638, 978729783, 1996000804, 1380088873, 1753334201,
                433326939, 1294775677
            ])
        );
    }

    #[test]
    fn sponge_known_answer() {
        let cap = safe_domain_separator::<9>(&[1, 2, 3, 4]);
        let input = [Fp::ONE; 5];
        let out = sponge::<8>(&cap, &input);
        assert_eq!(
            out,
            fe([
                477552014, 972552740, 1695413639, 12018845, 1258639896, 1015276872, 1156253900,
                190862312
            ])
        );
    }

    #[test]
    fn sponge_exact_multiple_has_no_extra_block() {
        let cap = [Fp::ONE; 9];
        let full = seq(15);
        let a = sponge::<8>(&cap, &full);
        // Manually: one absorb, then squeeze.
        let mut state = [Fp::ZERO; 24];
        state[..9].copy_from_slice(&cap);
        state[9..].copy_from_slice(&full);
        permute24(&mut state);
        assert_eq!(&a[..], &state[9..17]);
    }
}
