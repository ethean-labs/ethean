//! Tweakable hash `Th(P, T, M)` and hash chains over Poseidon1.
//!
//! Three input shapes, matching leanSpec `xmss/poseidon.py`:
//! one digest (chain step, width-16 compression), two digests (Merkle node,
//! width-24 compression), or many digests (leaf, width-24 sponge).

use super::params::{SchemeParams, CAPACITY, HASH_LEN, PARAM_LEN, TWEAK_LEN};
use super::tweak::Tweak;
use crate::field::Fp;
use crate::poseidon::{compress, safe_domain_separator, sponge, Poseidon24Input};

/// Hash digest: 8 field elements (32 bytes on the wire).
pub type Digest = [Fp; HASH_LEN];
/// Public parameter: 5 field elements (20 bytes on the wire).
pub type Parameter = [Fp; PARAM_LEN];

/// Hash a single digest: input `[message | parameter | tweak]`, width 16.
#[inline]
pub fn hash_one(parameter: &Parameter, tweak: Tweak, message: &Digest) -> Digest {
    let tweak_fe = tweak.to_field_elements();
    let mut input = [Fp::ZERO; HASH_LEN + PARAM_LEN + TWEAK_LEN];
    input[..HASH_LEN].copy_from_slice(message);
    input[HASH_LEN..HASH_LEN + PARAM_LEN].copy_from_slice(parameter);
    input[HASH_LEN + PARAM_LEN..].copy_from_slice(&tweak_fe);
    compress::<HASH_LEN>(Poseidon24Input::Width16, &input)
}

/// Hash two digests: input `[parameter | tweak | left | right]`, width 24.
#[inline]
pub fn hash_pair(parameter: &Parameter, tweak: Tweak, left: &Digest, right: &Digest) -> Digest {
    let tweak_fe = tweak.to_field_elements();
    let mut input = [Fp::ZERO; PARAM_LEN + TWEAK_LEN + 2 * HASH_LEN];
    input[..PARAM_LEN].copy_from_slice(parameter);
    input[PARAM_LEN..PARAM_LEN + TWEAK_LEN].copy_from_slice(&tweak_fe);
    input[PARAM_LEN + TWEAK_LEN..PARAM_LEN + TWEAK_LEN + HASH_LEN].copy_from_slice(left);
    input[PARAM_LEN + TWEAK_LEN + HASH_LEN..].copy_from_slice(right);
    compress::<HASH_LEN>(Poseidon24Input::Width24, &input)
}

/// Capacity value for leaf hashing; depends only on the scheme dimension.
pub fn leaf_capacity(params: &SchemeParams) -> [Fp; CAPACITY] {
    safe_domain_separator::<CAPACITY>(&[
        PARAM_LEN as u32,
        TWEAK_LEN as u32,
        params.dimension as u32,
        HASH_LEN as u32,
    ])
}

/// Hash all chain ends of an epoch into the Merkle leaf (sponge mode).
///
/// `capacity` must come from [`leaf_capacity`] for the same parameters.
pub fn hash_leaf(
    parameter: &Parameter,
    capacity: &[Fp; CAPACITY],
    epoch: u32,
    chain_ends: &[Digest],
) -> Digest {
    let tweak_fe = Tweak::Tree {
        level: 0,
        index: epoch,
    }
    .to_field_elements();
    let mut input = Vec::with_capacity(PARAM_LEN + TWEAK_LEN + chain_ends.len() * HASH_LEN);
    input.extend_from_slice(parameter);
    input.extend_from_slice(&tweak_fe);
    for end in chain_ends {
        input.extend_from_slice(end);
    }
    sponge::<HASH_LEN>(capacity, &input)
}

/// Generic dispatch by message count (leaf mode recomputes the capacity).
pub fn tweak_hash(
    params: &SchemeParams,
    parameter: &Parameter,
    tweak: Tweak,
    message: &[Digest],
) -> Digest {
    match message {
        [single] => hash_one(parameter, tweak, single),
        [left, right] => hash_pair(parameter, tweak, left, right),
        _ => {
            let Tweak::Tree { level: 0, index } = tweak else {
                panic!("leaf hashing requires a level-0 tree tweak");
            };
            hash_leaf(parameter, &leaf_capacity(params), index, message)
        }
    }
}

/// Walk `steps` chain steps from position `start_pos` in chain `chain_index`.
pub fn chain(
    parameter: &Parameter,
    epoch: u32,
    chain_index: u8,
    start_pos: u8,
    steps: usize,
    start: &Digest,
) -> Digest {
    let mut current = *start;
    for j in 0..steps {
        let pos = start_pos + j as u8 + 1;
        current = hash_one(
            parameter,
            Tweak::Chain {
                epoch,
                chain_index,
                pos,
            },
            &current,
        );
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    /// leanSpec `tests/spec/crypto/xmss/test_poseidon.py` chain-step vector.
    #[test]
    fn chain_step_known_answer() {
        let parameter = [Fp::ONE; PARAM_LEN];
        let digest: Digest = std::array::from_fn(|i| Fp::new(i as u32).unwrap());
        let out = hash_one(
            &parameter,
            Tweak::Chain {
                epoch: 0,
                chain_index: 1,
                pos: 1,
            },
            &digest,
        );
        let expected = [
            486628877, 1489818024, 465621198, 1039062572, 735121219, 2072497154, 800300299,
            543601961,
        ]
        .map(|v| Fp::new(v).unwrap());
        assert_eq!(out, expected);
    }

    #[test]
    fn chain_is_associative() {
        let parameter: Parameter = std::array::from_fn(|i| Fp::from_u64(i as u64 + 11));
        let start: Digest = std::array::from_fn(|i| Fp::from_u64(i as u64 * 77));
        let direct = chain(&parameter, 9, 3, 0, 7, &start);
        for split in 0..=7 {
            let mid = chain(&parameter, 9, 3, 0, split, &start);
            let end = chain(&parameter, 9, 3, split as u8, 7 - split, &mid);
            assert_eq!(direct, end);
        }
    }
}
