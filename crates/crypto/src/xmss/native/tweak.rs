//! Tweak and epoch encodings as base-`p` field-element limbs.

use super::params::{TWEAK_LEN, TWEAK_PREFIX_CHAIN, TWEAK_PREFIX_MESSAGE, TWEAK_PREFIX_TREE};
use crate::field::{base_p_limbs_from_u128, Fp};

/// Domain-separating address for a tweakable-hash call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tweak {
    /// Merkle node at `level` (leaves are level 0), position `index`.
    Tree { level: u8, index: u32 },
    /// Hash-chain step `pos` of chain `chain_index` in `epoch`.
    Chain {
        epoch: u32,
        chain_index: u8,
        pos: u8,
    },
}

impl Tweak {
    /// Pack as one integer, then split into `TWEAK_LEN` base-`p` limbs.
    pub fn to_field_elements(self) -> [Fp; TWEAK_LEN] {
        let packed: u128 = match self {
            Tweak::Tree { level, index } => {
                ((level as u128) << 40) | ((index as u128) << 8) | TWEAK_PREFIX_TREE as u128
            }
            Tweak::Chain {
                epoch,
                chain_index,
                pos,
            } => {
                ((epoch as u128) << 24)
                    | ((chain_index as u128) << 16)
                    | ((pos as u128) << 8)
                    | TWEAK_PREFIX_CHAIN as u128
            }
        };
        base_p_limbs_from_u128(packed).expect("56-bit tweak fits in two limbs")
    }
}

/// Epoch encoding used inside the message hash: `(epoch << 8) | 0x02`.
pub fn encode_epoch(epoch: u32) -> [Fp; TWEAK_LEN] {
    let packed = ((epoch as u128) << 8) | TWEAK_PREFIX_MESSAGE as u128;
    base_p_limbs_from_u128(packed).expect("40-bit epoch tweak fits in two limbs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::P;

    #[test]
    fn tweaks_are_distinct_and_little_endian_limbs() {
        let tree = Tweak::Tree { level: 1, index: 5 }.to_field_elements();
        let chain = Tweak::Chain {
            epoch: 0,
            chain_index: 1,
            pos: 1,
        }
        .to_field_elements();
        assert_ne!(tree, chain);
        let packed = (1u64 << 40) | (5 << 8) | 1;
        assert_eq!(tree[0].as_u32() as u64, packed % P as u64);
        assert_eq!(tree[1].as_u32() as u64, packed / P as u64);
        assert_eq!(encode_epoch(7)[0].as_u32(), (7 << 8) | 2);
        assert_eq!(encode_epoch(7)[1], Fp::ZERO);
    }
}
