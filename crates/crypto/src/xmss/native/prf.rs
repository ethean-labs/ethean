//! SHAKE128-based PRF producing chain start digests and encoding randomness.

use super::params::{HASH_LEN, MESSAGE_LEN, PRF_KEY_LEN, RAND_LEN};
use crate::field::Fp;
use crate::keccak::Shake128;

const DOMAIN_SEP: [u8; 16] = [
    0xae, 0xae, 0x22, 0xff, 0x00, 0x01, 0xfa, 0xff, 0x21, 0xaf, 0x12, 0x00, 0x01, 0x11, 0xff, 0x00,
];
const SUBDOMAIN_CHAIN_START: u8 = 0x00;
const SUBDOMAIN_RANDOMNESS: u8 = 0x01;
/// Bytes squeezed per field element; interpreted big-endian, reduced mod `p`.
const BYTES_PER_FE: usize = 16;

/// PRF key: 32 uniformly random bytes.
pub type PrfKey = [u8; PRF_KEY_LEN];

fn squeeze_field_elements<const N: usize>(xof: &mut Shake128) -> [Fp; N] {
    let mut out = [Fp::ZERO; N];
    let mut buf = [0u8; BYTES_PER_FE];
    for fe in out.iter_mut() {
        xof.squeeze(&mut buf);
        *fe = Fp::from_u128(u128::from_be_bytes(buf));
    }
    out
}

/// Chain start for `(epoch, chain_index)`.
pub fn chain_start(key: &PrfKey, epoch: u32, chain_index: u64) -> [Fp; HASH_LEN] {
    let mut xof = Shake128::new();
    xof.update(&DOMAIN_SEP);
    xof.update(&[SUBDOMAIN_CHAIN_START]);
    xof.update(key);
    xof.update(&epoch.to_be_bytes());
    xof.update(&chain_index.to_be_bytes());
    squeeze_field_elements(&mut xof)
}

/// Encoding randomness `rho` for `(epoch, message, counter)`.
pub fn randomness(
    key: &PrfKey,
    epoch: u32,
    message: &[u8; MESSAGE_LEN],
    counter: u64,
) -> [Fp; RAND_LEN] {
    let mut xof = Shake128::new();
    xof.update(&DOMAIN_SEP);
    xof.update(&[SUBDOMAIN_RANDOMNESS]);
    xof.update(key);
    xof.update(&epoch.to_be_bytes());
    xof.update(message);
    xof.update(&counter.to_be_bytes());
    squeeze_field_elements(&mut xof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_separated() {
        let key = [7u8; 32];
        let a = chain_start(&key, 3, 1);
        assert_eq!(a, chain_start(&key, 3, 1));
        assert_ne!(a, chain_start(&key, 3, 2));
        assert_ne!(a, chain_start(&key, 4, 1));
        let msg = [1u8; 32];
        let r = randomness(&key, 3, &msg, 0);
        assert_ne!(r, randomness(&key, 3, &msg, 1));
        assert_ne!(r, randomness(&[8u8; 32], 3, &msg, 0));
    }
}
