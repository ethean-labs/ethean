//! Req/resp handler scaffolding (Status + block retrieval policy).

mod blocks_by_root;
mod blocks_outbound;
mod blocks_response;
mod handler;
mod status_outbound;
mod status_session;
mod tracker;

pub use blocks_by_root::{
    blocks_by_root_for_status_gap, blocks_by_root_protocol_id, encode_blocks_by_root,
};
pub use blocks_outbound::{prepare_blocks_by_root_outbound, OutboundBlocksByRootRequest};
pub use blocks_response::{decode_blocks_by_root_response, encode_blocks_by_root_response};
pub use handler::{handle_status, StatusExchange};
pub use status_outbound::{prepare_status_outbounds, OutboundStatusRequest};
pub use status_session::StatusSessionBook;
pub use tracker::{RequestId, RequestTracker};
