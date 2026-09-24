//! Lean Consensus wire codecs: topics, Snappy, Status, req/resp.

#![forbid(unsafe_code)]

pub mod codec;
pub mod error;
pub mod fork_id;
pub mod gossip;
pub mod limits;
pub mod message_id;
pub mod reqresp;
pub mod snappy;
pub mod status;
pub mod topics;
pub mod varint;

pub use codec::{
    decode_request, decode_response, decode_response_chunk, decode_response_stream,
    encode_request, encode_response, encode_response_stream, split_one_chunk, ResponseChunk,
};
pub use error::{Result, WireError};
pub use fork_id::{
    fork_identifier_bytes, fork_segment_from_name, fork_segment_hex, fork_segment_resolve,
    normalize_digest_hex, parse_digest_hex, LSTAR_GOSSIP_DIGEST,
};
pub use limits::{
    max_compressed_len, max_message_size, MAX_BLOCKS_PER_REQUEST, MAX_COMPRESSED_GOSSIP_BYTES,
    MAX_DECOMPRESSED_BYTES, MAX_ERROR_MESSAGE_SIZE, MAX_MESSAGE_SIZE, MAX_PAYLOAD_SIZE,
    MAX_SNAPPY_EXPANSION_RATIO, MAX_STREAMS_PER_PEER, STATUS_TIMEOUT_MS,
};
pub use message_id::{
    compute_message_id, message_id, message_id_from_raw, message_id_invalid_snappy,
    message_id_valid_snappy, MessageId, MESSAGE_DOMAIN_INVALID_SNAPPY, MESSAGE_DOMAIN_VALID_SNAPPY,
};
pub use reqresp::{BlocksByRangeRequest, BlocksByRootRequest, ResponseCode};
pub use snappy::{
    compress_frame, compress_framed, compress_raw, decompress_frame, decompress_framed,
    decompress_raw,
};
pub use status::{Checkpoint, Status};
pub use topics::{
    rpc_blocks_by_range, rpc_blocks_by_root, rpc_status, topic_aggregation, topic_attestation,
    topic_block,
};
pub use varint::{decode_varint, encode_varint};
