//! Lean Consensus HTTP API (`/lean/v0` hive surface; `/lean/v1` aliases).

#![forbid(unsafe_code)]

pub mod admin;
pub mod auth;
pub mod dto;
pub mod error;
pub mod events;
pub mod handlers;
pub mod hexutil;
pub mod http;
pub mod json_body;
pub mod limits;
pub mod parse;
pub mod routes;
pub mod server;
pub mod state;
pub mod test_driver;
pub mod view;

#[cfg(test)]
mod http_test;

pub use admin::request_shutdown;
pub use auth::{authorize_admin, validate_admin_token, BindScope};
pub use dto::{
    AggregatorStatusBody, AggregatorToggleBody, CheckpointBody, DutiesView, DutyRow, FinalizedView,
    ForkChoiceBody, ForkChoiceStatsView, ForkChoiceNodeBody, HeadView, HealthBody, SyncView,
};
pub use error::{Result, RpcError};
pub use events::{AdminEvent, EventBuffer};
pub use hexutil::hex_root_0x;
pub use http::spawn_lean_http;
pub use limits::{ADMIN_RPS, MAX_BODY_BYTES, MAX_EVENT_BACKLOG, PUBLIC_RPS};
pub use routes::{match_route, requires_admin, Route};
pub use server::{dispatch, IncomingRequest};
pub use state::{hex_root, ApiSnapshot, SharedApiState};
pub use test_driver::{DriverHandle, TestDriver};
pub use view::ForkChoiceView;
