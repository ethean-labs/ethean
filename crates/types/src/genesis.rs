//! Genesis configuration committed into consensus state.

use ethean_ssz::{decode_u64, encode_u64, expect_exhausted, hash_tree_root_u64, Root};

use crate::error::TypesError;

/// Chain configuration stored in [`crate::State`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GenesisConfig {
    pub genesis_time: u64,
}

impl GenesisConfig {
    pub fn new(genesis_time: u64) -> Self {
        Self { genesis_time }
    }

    pub fn ssz_encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8);
        encode_u64(&mut out, self.genesis_time);
        out
    }

    pub fn ssz_decode(input: &[u8]) -> Result<Self, TypesError> {
        let mut c = 0;
        let genesis_time = decode_u64(input, &mut c)?;
        expect_exhausted(input, c)?;
        Ok(Self { genesis_time })
    }

    pub fn hash_tree_root(&self) -> Root {
        hash_tree_root_u64(self.genesis_time)
    }
}
