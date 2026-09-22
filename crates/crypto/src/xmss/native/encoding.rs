//! Message hashing and the aborting target-sum incomparable encoding.
//!
//! `H(msg, P, epoch, rho)` is a width-24 Poseidon compression whose first
//! `ceil(v / Z)` outputs are rejection-sampled and split into base-`w`
//! digits (HHKTW26 §6.1). Signing keeps resampling `rho` until the digits
//! sum to the target.

use super::params::{SchemeParams, BASE, MESSAGE_LEN, MSG_LEN, Q, RAND_LEN, Z};
use super::tweak::encode_epoch;
use super::tweak_hash::Parameter;
use crate::field::{base_p_limbs_from_le_bytes, Fp};
use crate::poseidon::{compress, Poseidon24Input};

/// Encoding randomness: 7 field elements (28 bytes on the wire).
pub type Randomness = [Fp; RAND_LEN];

/// Rejection threshold `Q * w^Z == p - 1`.
const THRESHOLD: u64 = Q as u64 * (BASE as u64).pow(Z);

/// Message bytes as a little-endian integer in 9 base-`p` limbs.
pub fn encode_message(message: &[u8; MESSAGE_LEN]) -> [Fp; MSG_LEN] {
    base_p_limbs_from_le_bytes::<MSG_LEN>(message).expect("256-bit message fits in 9 limbs")
}

/// Raw message hash: `compress24([message | parameter | epoch | rho])`,
/// truncated to the scheme's `message_hash_len()`.
pub fn message_hash_elements(
    params: &SchemeParams,
    parameter: &Parameter,
    epoch: u32,
    rho: &Randomness,
    message: &[u8; MESSAGE_LEN],
) -> Vec<Fp> {
    let msg_fe = encode_message(message);
    let epoch_fe = encode_epoch(epoch);
    let mut input = Vec::with_capacity(23);
    input.extend_from_slice(&msg_fe);
    input.extend_from_slice(parameter);
    input.extend_from_slice(&epoch_fe);
    input.extend_from_slice(rho);
    let out = compress::<8>(Poseidon24Input::Width24, &input);
    out[..params.message_hash_len()].to_vec()
}

/// Rejection-sample each element and split into base-`w` digits.
/// Returns `None` if any element is at or above the threshold.
pub fn aborting_decode(params: &SchemeParams, elements: &[Fp]) -> Option<Vec<u8>> {
    let mut digits = Vec::with_capacity(params.message_hash_len() * Z as usize);
    for fe in elements {
        let value = fe.as_u32() as u64;
        if value >= THRESHOLD {
            return None;
        }
        let mut quotient = value / Q as u64;
        for _ in 0..Z {
            digits.push((quotient % BASE as u64) as u8);
            quotient /= BASE as u64;
        }
    }
    digits.truncate(params.dimension);
    Some(digits)
}

/// Full target-sum encoding; `None` on abort or sum mismatch.
pub fn target_sum_encode(
    params: &SchemeParams,
    parameter: &Parameter,
    epoch: u32,
    rho: &Randomness,
    message: &[u8; MESSAGE_LEN],
) -> Option<Vec<u8>> {
    let elements = message_hash_elements(params, parameter, epoch, rho, message);
    let digits = aborting_decode(params, &elements)?;
    let sum: u32 = digits.iter().map(|&d| d as u32).sum();
    (sum == params.target_sum).then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::super::params::{PROD, TEST};
    use super::*;
    use crate::field::P;

    #[test]
    fn threshold_is_p_minus_one() {
        assert_eq!(THRESHOLD, P as u64 - 1);
        assert!(aborting_decode(&PROD, &[Fp::MAX; 6]).is_none());
    }

    #[test]
    fn decode_digit_order_is_lsb_first() {
        // value = Q * d with d = 0o7654_3210 -> digits [0,1,2,3,4,5,6,7].
        let d: u64 = 0o76543210;
        let fe = Fp::from_u64(d * Q as u64);
        let digits = aborting_decode(&PROD, &[fe; 6]).unwrap();
        assert_eq!(&digits[..8], &[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(digits.len(), PROD.dimension);
        assert_eq!(aborting_decode(&TEST, &[fe]).unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn encoding_eventually_succeeds_for_test_params() {
        let parameter: Parameter = std::array::from_fn(|i| Fp::from_u64(i as u64 + 1));
        let message = [0x5au8; 32];
        let mut found = false;
        for counter in 0..2000u64 {
            let rho = super::super::prf::randomness(&[3u8; 32], 5, &message, counter);
            if let Some(digits) = target_sum_encode(&TEST, &parameter, 5, &rho, &message) {
                assert_eq!(
                    digits.iter().map(|&d| d as u32).sum::<u32>(),
                    TEST.target_sum
                );
                assert!(digits.iter().all(|&d| (d as u32) < BASE));
                found = true;
                break;
            }
        }
        assert!(found, "no target-sum codeword within 2000 tries");
    }
}
