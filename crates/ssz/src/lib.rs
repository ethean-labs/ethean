//! Minimal SSZ for Ethean Lean Consensus containers.
//!
//! Phase 00 leanSpec pins `eth-ssz-specs` for Python. This crate is the
//! in-tree Rust codec used by `ethean-types`; it does not wrap peer clients.

#![forbid(unsafe_code)]

mod decode;
mod encode;
mod error;
mod tree_hash;

pub use decode::{
    decode_bitlist, decode_bool, decode_fixed_bytes, decode_offset_list, decode_u16, decode_u32,
    decode_u64, decode_u8, expect_exhausted, need,
};
pub use encode::{
    encode_bitlist, encode_bool, encode_byte_list, encode_fixed_bytes, encode_offset_list,
    encode_u16, encode_u32, encode_u64, encode_u8,
};
pub use error::SszError;
pub use tree_hash::{
    chunk_from_bytes, hash_nodes, hash_tree_root_bitlist, hash_tree_root_bool,
    hash_tree_root_bytes, hash_tree_root_container, hash_tree_root_list, hash_tree_root_u64,
    merkleize, mix_in_length, Root, ZERO_CHUNK,
};
