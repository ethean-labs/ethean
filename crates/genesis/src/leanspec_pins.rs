//! leanSpec prod-scheme genesis root pins (small validator sets).

use ethean_primitives::Bytes52;
use ethean_ssz::Root;

use crate::builder::{BuiltGenesis, GenesisBuilder};

/// Prod-scheme attestation/proposal key pair 0 from leanSpec fill fixtures.
const PROD_V0_ATT: &str = "6c9dc94cb124fd7eac91680ca6f7c131d7b02f1629ca842b0e714d557da1276b2e5f3d110aab9d6eb8c10c7d1ba0f93cd849c82e";
const PROD_V0_PROP: &str =
    "4f0098552e05e766afeb0d021742f626de106215bd0c5a5d2516fc75dc3ffc121159b63b6664354dc4eb353631b54a5ad0cc7b14";
const PROD_V1_ATT: &str = "adc304243eb9f60ad1016e6610f11420dba97e450171971dda35bf50d4f14117ad3b9f06083c9d05f73d5e12f9e9ce1e7aa65866";
const PROD_V1_PROP: &str =
    "e2508c4edcc47c50505281470653e6555fe9785380e97c427e99b677f949b125d17f2c58a45c1b5349f20c27db10ea78d2fceb37";
const PROD_V2_ATT: &str = "44095a46c1dd521b2e37c626b3401045f2a41713b034d95e9888fb34a15c280a8045b7043b6cca1c3353a6089ae5aa027c548d4b";
const PROD_V2_PROP: &str =
    "d72b9b4bf39f805d637c855bb3266b373aa97e262cea76359b84e70805268b520cbe62284be2f81da2069b0c5298912169fa935f";
const PROD_V3_ATT: &str = "c6ce413056aff1787a8d01575b11781c08ab595fea7cb46372fd8376e054f829da06c251d8e13643e751d9737d52e4474fddab5c";
const PROD_V3_PROP: &str =
    "d763f750d84c1840a6e2fb16b9f5e13f372e1f4646588a0d0ab96e483b42161f47858d5acb6d3a6bdae7e65a672cd2233fe77c16";

/// `hash_tree_root(State)` for genesis_time=0, four prod-scheme validators
/// (header `state_root` still zero — leanSpec `generate_genesis` shape).
pub const PROD4_GENESIS_STATE_ROOT: Root = [
    0xf5, 0x7f, 0x32, 0x6a, 0x07, 0x0f, 0x9b, 0x50, 0x36, 0x29, 0x26, 0x65, 0x56, 0x48, 0x9c, 0xd3,
    0x64, 0x63, 0xdb, 0xd8, 0xe8, 0x0e, 0x60, 0xc8, 0x56, 0x7b, 0xc9, 0x71, 0xab, 0x17, 0x19, 0x19,
];

/// Sealed genesis header root (`state_root` cached) for the same four validators.
/// Matches leanSpec `test_first_post_genesis_block_sets_checkpoint_anchor_roots`
/// slot-1 `parentRoot`.
pub const PROD4_GENESIS_BLOCK_ROOT: Root = [
    0x7a, 0xbe, 0x55, 0x4d, 0xa9, 0x98, 0x33, 0x9e, 0xdf, 0xd2, 0xac, 0x2b, 0x86, 0x87, 0x4b, 0x1e,
    0xeb, 0x20, 0x0f, 0x91, 0xf1, 0x1a, 0x65, 0x70, 0x1b, 0xfb, 0xba, 0xdc, 0x65, 0xe1, 0xf8, 0x62,
];

/// Sealed genesis header root for a single prod-scheme validator (index 0).
/// Matches leanSpec `test_genesis_single_validator` slot-1 `parentRoot`.
pub const PROD1_GENESIS_BLOCK_ROOT: Root = [
    0xc9, 0x5a, 0xb2, 0x7e, 0x70, 0xa5, 0x63, 0x78, 0xd4, 0x27, 0x71, 0x88, 0x60, 0x4e, 0xcf, 0xad,
    0x53, 0x60, 0x95, 0xa9, 0xc2, 0x45, 0x57, 0x63, 0x41, 0xa1, 0xd7, 0x63, 0xa7, 0x2f, 0x75, 0x6d,
];

fn parse52(hex: &str) -> Bytes52 {
    let mut out = [0u8; 52];
    for i in 0..52 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("hex");
    }
    Bytes52(out)
}

fn prod_keys(n: usize) -> Vec<(Bytes52, Bytes52)> {
    let all = [
        (PROD_V0_ATT, PROD_V0_PROP),
        (PROD_V1_ATT, PROD_V1_PROP),
        (PROD_V2_ATT, PROD_V2_PROP),
        (PROD_V3_ATT, PROD_V3_PROP),
    ];
    all.iter()
        .take(n)
        .map(|(a, p)| (parse52(a), parse52(p)))
        .collect()
}

/// Build leanSpec-shaped genesis (time 0, first `n` prod-scheme validators).
pub fn prod_scheme_genesis(n: usize) -> BuiltGenesis {
    GenesisBuilder::new(0)
        .with_validator_keys(prod_keys(n))
        .build()
        .expect("prod-scheme genesis")
}

/// Cache `state_root` into the header (leanSpec `process_slots` / seal rule).
pub fn seal_genesis_header(built: &BuiltGenesis) -> Root {
    let mut header = built.state.latest_block_header.clone();
    header.state_root = built.state_root;
    header.hash_tree_root()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::EMPTY_BLOCK_BODY_ROOT;

    #[test]
    fn prod4_state_root_pinned() {
        let built = prod_scheme_genesis(4);
        assert_eq!(built.state.validators.len(), 4);
        assert_eq!(built.state.latest_block_header.body_root, EMPTY_BLOCK_BODY_ROOT);
        assert_eq!(built.state_root, PROD4_GENESIS_STATE_ROOT);
    }

    #[test]
    fn prod4_sealed_header_matches_leanspec_parent_root() {
        let built = prod_scheme_genesis(4);
        assert_eq!(seal_genesis_header(&built), PROD4_GENESIS_BLOCK_ROOT);
    }

    #[test]
    fn prod1_sealed_header_matches_leanspec_parent_root() {
        let built = prod_scheme_genesis(1);
        assert_eq!(seal_genesis_header(&built), PROD1_GENESIS_BLOCK_ROOT);
    }
}
