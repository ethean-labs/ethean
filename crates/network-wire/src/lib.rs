//! Lean Consensus wire codecs: topics, Snappy, Status, req/resp shapes.

#![forbid(unsafe_code)]

pub mod error;
pub mod fork_id;
pub mod limits;
pub mod message_id;
pub mod reqresp;
pub mod snappy;
pub mod status;
pub mod topics;

pub use error::{Result, WireError};
pub use fork_id::{fork_identifier_bytes, fork_segment_hex};
pub use limits::{
    MAX_BLOCKS_PER_REQUEST, MAX_COMPRESSED_GOSSIP_BYTES, MAX_DECOMPRESSED_BYTES,
    MAX_SNAPPY_EXPANSION_RATIO, MAX_STREAMS_PER_PEER, STATUS_TIMEOUT_MS,
};
pub use message_id::message_id;
pub use reqresp::{BlocksByRangeRequest, BlocksByRootRequest, ResponseCode};
pub use snappy::{compress_framed, compress_raw, decompress_framed, decompress_raw};
pub use status::Status;
pub use topics::{
    fork_segment_from_name, rpc_blocks_by_range, rpc_blocks_by_root, rpc_status, topic_aggregation,
    topic_attestation, topic_block, FORBIDDEN_DUMMY_FORK,
};
