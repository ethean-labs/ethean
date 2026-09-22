//! Randomness sources for key generation and Merkle padding.

use super::params::PRF_KEY_LEN;
use super::tweak_hash::{Digest, Parameter};
use crate::error::{CryptoError, Result};
use crate::field::Fp;
use crate::keccak::Shake128;

/// Source of random bytes.
pub trait RandomSource {
    fn fill(&mut self, out: &mut [u8]) -> Result<()>;
}

/// Field-element helpers over any [`RandomSource`].
pub trait RandomExt: RandomSource {
    fn field_elements<const N: usize>(&mut self) -> Result<[Fp; N]> {
        let mut buf = [0u8; 16];
        let mut out = [Fp::ZERO; N];
        for fe in out.iter_mut() {
            self.fill(&mut buf)?;
            *fe = Fp::from_u128(u128::from_be_bytes(buf));
        }
        Ok(out)
    }

    fn parameter(&mut self) -> Result<Parameter> {
        self.field_elements()
    }

    fn digest(&mut self) -> Result<Digest> {
        self.field_elements()
    }

    fn prf_key(&mut self) -> Result<[u8; PRF_KEY_LEN]> {
        let mut key = [0u8; PRF_KEY_LEN];
        self.fill(&mut key)?;
        Ok(key)
    }
}

impl<R: RandomSource + ?Sized> RandomExt for R {}

/// Operating-system entropy (`/dev/urandom`).
#[derive(Debug, Default, Clone, Copy)]
pub struct OsRandom;

impl RandomSource for OsRandom {
    fn fill(&mut self, out: &mut [u8]) -> Result<()> {
        use std::io::Read;
        let mut file = std::fs::File::open("/dev/urandom")
            .map_err(|e| CryptoError::RandomnessUnavailable(e.to_string()))?;
        file.read_exact(out)
            .map_err(|e| CryptoError::RandomnessUnavailable(e.to_string()))
    }
}

/// Deterministic SHAKE128 stream from a seed; for tests and reproducible keys.
pub struct SeededRandom {
    xof: Shake128,
}

impl SeededRandom {
    pub fn new(seed: &[u8]) -> Self {
        let mut xof = Shake128::new();
        xof.update(b"ethean-xmss-seeded-random/v1");
        xof.update(seed);
        Self { xof }
    }
}

impl RandomSource for SeededRandom {
    fn fill(&mut self, out: &mut [u8]) -> Result<()> {
        self.xof.squeeze(out);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_random_and_seeded_behave() {
        let a = OsRandom.prf_key().unwrap();
        let b = OsRandom.prf_key().unwrap();
        assert_ne!(a, b);
        let x = SeededRandom::new(b"seed").prf_key().unwrap();
        let y = SeededRandom::new(b"seed").prf_key().unwrap();
        assert_eq!(x, y);
        assert_ne!(x, SeededRandom::new(b"other").prf_key().unwrap());
    }
}
