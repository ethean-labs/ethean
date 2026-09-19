//! Gossip codec helpers wrapping network-wire Snappy.

use ethean_network_wire::{compress_raw, decompress_raw, Result as WireResult};

/// Encode a gossip SSZ payload with raw Snappy.
pub fn encode_gossip(ssz: &[u8]) -> WireResult<Vec<u8>> {
    compress_raw(ssz)
}

/// Decode a gossip payload from raw Snappy.
pub fn decode_gossip(compressed: &[u8]) -> WireResult<Vec<u8>> {
    decompress_raw(compressed)
}
