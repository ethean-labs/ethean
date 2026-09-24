//! Wire size and decompression budgets (leanSpec lstar networking).

/// Maximum uncompressed payload size (leanSpec `MAX_PAYLOAD_SIZE`, 10 MiB).
pub const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Maximum compressed gossip / req-resp frame (snappy worst case + 1 KiB).
///
/// `max(32 + n + n/6 + 1024, 1 MiB)` with `n = MAX_PAYLOAD_SIZE`, matching
/// ream `max_message_size()`.
pub const MAX_MESSAGE_SIZE: usize = {
    let n = MAX_PAYLOAD_SIZE as u64;
    let framed = 32 + n + n / 6 + 1024;
    if framed > 1024 * 1024 {
        framed as usize
    } else {
        1024 * 1024
    }
};

/// Maximum compressed gossip payload before decompress.
pub const MAX_COMPRESSED_GOSSIP_BYTES: usize = MAX_MESSAGE_SIZE;

/// Maximum decompressed gossip / SSZ payload.
pub const MAX_DECOMPRESSED_BYTES: usize = MAX_PAYLOAD_SIZE;

/// Maximum compressed / expanded ratio (bomb guard for raw gossip).
pub const MAX_SNAPPY_EXPANSION_RATIO: usize = 10;

/// Maximum blocks in a single blocks_by_root / range request.
pub const MAX_BLOCKS_PER_REQUEST: u64 = 1024;

/// Maximum concurrent req/resp streams per peer (soft policy).
pub const MAX_STREAMS_PER_PEER: usize = 8;

/// Status handshake timeout in milliseconds (leanSpec `RESP_TIMEOUT`).
pub const STATUS_TIMEOUT_MS: u64 = 10_000;

/// Maximum UTF-8 error message in a failed req/resp chunk.
pub const MAX_ERROR_MESSAGE_SIZE: usize = 256;

/// Worst-case snappy compressed length for payload size `n`.
pub fn max_compressed_len(n: u64) -> u64 {
    32 + n + n / 6
}

/// Worst-case framed message size (snappy + 1 KiB overhead, at least 1 MiB).
pub fn max_message_size() -> usize {
    MAX_MESSAGE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_cap_is_1024() {
        assert_eq!(MAX_BLOCKS_PER_REQUEST, 1024);
    }

    #[test]
    fn payload_is_10_mib() {
        assert_eq!(MAX_PAYLOAD_SIZE, 10 * 1024 * 1024);
        assert!(MAX_MESSAGE_SIZE > MAX_PAYLOAD_SIZE);
        assert_eq!(
            MAX_MESSAGE_SIZE,
            (max_compressed_len(MAX_PAYLOAD_SIZE as u64) + 1024) as usize
        );
    }
}
