//! Wire size and decompression budgets (Phase 10).

/// Maximum compressed gossip payload before decompress.
pub const MAX_COMPRESSED_GOSSIP_BYTES: usize = 2 * 1024 * 1024;

/// Maximum decompressed gossip / SSZ payload.
pub const MAX_DECOMPRESSED_BYTES: usize = 2 * 1024 * 1024;

/// Maximum compressed / expanded ratio (bomb guard).
pub const MAX_SNAPPY_EXPANSION_RATIO: usize = 10;

/// Maximum blocks in a single blocks_by_root / range request.
pub const MAX_BLOCKS_PER_REQUEST: u64 = 1024;

/// Maximum concurrent req/resp streams per peer (soft policy).
pub const MAX_STREAMS_PER_PEER: usize = 8;

/// Status handshake timeout in milliseconds (policy; transport wires later).
pub const STATUS_TIMEOUT_MS: u64 = 10_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_cap_is_1024() {
        assert_eq!(MAX_BLOCKS_PER_REQUEST, 1024);
    }
}
