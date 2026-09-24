//! Lean Consensus HTTP API surface (no Beacon /eth/v1 compatibility).

#![forbid(unsafe_code)]

pub mod admin;
pub mod auth;
pub mod dto;
pub mod error;
pub mod events;
pub mod http;
pub mod json_body;
pub mod limits;
pub mod routes;
pub mod server;
pub mod state;

pub use admin::request_shutdown;
pub use auth::{authorize_admin, validate_admin_token, BindScope};
pub use dto::{FinalizedView, ForkChoiceView, HeadView, SyncView};
pub use error::{Result, RpcError};
pub use events::{AdminEvent, EventBuffer};
pub use http::spawn_lean_http;
pub use limits::{ADMIN_RPS, MAX_BODY_BYTES, MAX_EVENT_BACKLOG, PUBLIC_RPS};
pub use routes::{match_route, requires_admin, Route};
pub use server::{dispatch, IncomingRequest};
pub use state::{hex_root, ApiSnapshot, SharedApiState};
