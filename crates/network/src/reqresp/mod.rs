//! Req/resp handler scaffolding (Status + block retrieval policy).

mod blocks_by_root;
mod handler;
mod status_session;
mod tracker;

pub use blocks_by_root::{
    blocks_by_root_for_status_gap, blocks_by_root_protocol_id, encode_blocks_by_root,
};
pub use handler::{handle_status, StatusExchange};
pub use status_session::StatusSessionBook;
pub use tracker::RequestTracker;
