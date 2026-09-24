//! Route handlers for `/lean/v0` (hive) and `/lean/v1` (operator aliases).

use crate::dto::{
    AggregatorStatusBody, AggregatorToggleBody, AggregatorToggleRequest, HealthBody,
};
use crate::error::RpcError;
use crate::json_body::{
    duties_json, events_json, finalized_json, fork_choice_json, head_json, identity_json, sync_json,
};
use crate::routes::Route;
use crate::state::SharedApiState;

/// HTTP response produced by a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpReply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

impl HttpReply {
    pub fn json(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.into().into_bytes(),
        }
    }

    pub fn ssz(bytes: Vec<u8>) -> Self {
        Self {
            status: 200,
            content_type: "application/octet-stream",
            body: bytes,
        }
    }

    pub fn text(status: u16, content_type: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type,
            body: body.into().into_bytes(),
        }
    }
}

const HEALTHY: &str = "healthy";
const SERVICE: &str = "lean-rpc-api";

/// Dispatch a matched route against the shared snapshot.
pub fn handle_route(route: Route, state: &SharedApiState, body: &[u8]) -> HttpReply {
    match route {
        Route::Health => {
            let v = HealthBody {
                status: HEALTHY.into(),
                service: SERVICE.into(),
                version: env!("CARGO_PKG_VERSION").into(),
            };
            HttpReply::json(200, serde_json::to_string(&v).unwrap_or_else(|_| "{}".into()))
        }
        Route::Ready => {
            if state.is_ready() {
                HttpReply::text(200, "text/plain; charset=utf-8", "ready\n")
            } else {
                HttpReply::text(503, "text/plain; charset=utf-8", "not ready\n")
            }
        }
        Route::NodeIdentity => {
            let mut snap = state.snapshot();
            snap.ready = state.is_ready();
            json_ok(identity_json(&snap))
        }
        Route::ChainHead => json_ok(head_json(&state.snapshot().head)),
        Route::ChainFinalized => json_ok(finalized_json(&state.snapshot().finalized)),
        Route::ChainSync => json_ok(sync_json(&state.snapshot().sync)),
        Route::ChainForkChoice => json_ok(fork_choice_json(&state.snapshot().fork_choice_stats)),
        Route::ValidatorDuties => json_ok(duties_json(&state.snapshot().duties)),
        Route::AdminShutdown => {
            state.request_shutdown();
            HttpReply::json(200, r#"{"ok":true}"#)
        }
        Route::AdminEvents => {
            let drained = state.drain_events(64);
            let pending = state.events_pending();
            json_ok(events_json(&drained, pending))
        }
        Route::CheckpointsJustified => checkpoints_justified(state),
        Route::ForkChoice => fork_choice(state),
        Route::StatesFinalized => ssz_or_missing(state, true),
        Route::BlocksFinalized => ssz_or_missing(state, false),
        Route::AggregatorGet => {
            let v = AggregatorStatusBody {
                is_aggregator: state.is_aggregator(),
            };
            HttpReply::json(200, serde_json::to_string(&v).unwrap_or_else(|_| "{}".into()))
        }
        Route::AggregatorPost => aggregator_post(state, body),
        Route::Metrics => HttpReply::text(
            404,
            "text/plain; charset=utf-8",
            "metrics live on :9100/metrics\n",
        ),
        Route::DriverForkChoiceInit
        | Route::DriverForkChoiceStep
        | Route::DriverStateTransition
        | Route::DriverVerifySignatures => crate::test_driver::handle_driver(route, state, body),
    }
}

fn json_ok(v: serde_json::Value) -> HttpReply {
    HttpReply::json(200, v.to_string())
}

fn checkpoints_justified(state: &SharedApiState) -> HttpReply {
    let snap = state.snapshot();
    if !snap.fork_choice.chain_ready {
        return error_json(RpcError::Unavailable("chain not ready".into()));
    }
    let body = serde_json::to_string(&snap.fork_choice.fork_choice.justified)
        .unwrap_or_else(|_| "{}".into());
    HttpReply::json(200, body)
}

fn fork_choice(state: &SharedApiState) -> HttpReply {
    let snap = state.snapshot();
    if !snap.fork_choice.chain_ready {
        return error_json(RpcError::Unavailable("chain not ready".into()));
    }
    let body =
        serde_json::to_string(&snap.fork_choice.fork_choice).unwrap_or_else(|_| "{}".into());
    HttpReply::json(200, body)
}

fn ssz_or_missing(state: &SharedApiState, state_bytes: bool) -> HttpReply {
    let snap = state.snapshot();
    if !snap.fork_choice.chain_ready {
        return error_json(RpcError::Unavailable("chain not ready".into()));
    }
    let blob = if state_bytes {
        snap.fork_choice.finalized_state_ssz
    } else {
        snap.fork_choice.finalized_block_ssz
    };
    match blob {
        Some(bytes) if !bytes.is_empty() => HttpReply::ssz(bytes),
        _ => error_json(RpcError::UnknownRoute("finalized payload missing".into())).with_status(404),
    }
}

fn aggregator_post(state: &SharedApiState, body: &[u8]) -> HttpReply {
    let parsed: Result<AggregatorToggleRequest, _> = serde_json::from_slice(body);
    match parsed {
        Ok(req) => {
            let previous = state.swap_aggregator(req.enabled);
            let v = AggregatorToggleBody {
                is_aggregator: req.enabled,
                previous,
            };
            HttpReply::json(200, serde_json::to_string(&v).unwrap_or_else(|_| "{}".into()))
        }
        Err(_) => error_json(RpcError::BadRequest(
            "body must be JSON {\"enabled\": bool}".into(),
        )),
    }
}

pub fn error_json(err: RpcError) -> HttpReply {
    let status = match &err {
        RpcError::UnknownRoute(_) => 404,
        RpcError::BodyTooLarge { .. } | RpcError::BadRequest(_) => 400,
        RpcError::Unauthorized => 401,
        RpcError::Forbidden => 403,
        RpcError::Unavailable(_) => 503,
    };
    let body = serde_json::json!({ "error": err.to_string() }).to_string();
    HttpReply::json(status, body)
}

trait WithStatus {
    fn with_status(self, status: u16) -> Self;
}

impl WithStatus for HttpReply {
    fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::ForkChoiceView;
    use ethean_primitives::HASH32_ZERO;

    #[test]
    fn health_is_hive_json() {
        let st = SharedApiState::new("");
        let r = handle_route(Route::Health, &st, b"");
        assert_eq!(r.status, 200);
        assert_eq!(r.content_type, "application/json");
        let v: HealthBody = serde_json::from_slice(&r.body).unwrap();
        assert_eq!(v.status, "healthy");
        assert_eq!(v.service, "lean-rpc-api");
        assert_eq!(v.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn justified_503_until_ready() {
        let st = SharedApiState::new("");
        let r = handle_route(Route::CheckpointsJustified, &st, b"");
        assert_eq!(r.status, 503);
    }

    #[test]
    fn aggregator_toggle_round_trip() {
        let st = SharedApiState::new("");
        let r = handle_route(Route::AggregatorPost, &st, br#"{"enabled":true}"#);
        assert_eq!(r.status, 200);
        let v: AggregatorToggleBody = serde_json::from_slice(&r.body).unwrap();
        assert!(v.is_aggregator);
        assert!(!v.previous);
        let bad = handle_route(Route::AggregatorPost, &st, b"not-json");
        assert_eq!(bad.status, 400);
    }

    #[test]
    fn ssz_octet_stream_when_published() {
        let st = SharedApiState::new("");
        let mut snap = st.snapshot();
        snap.fork_choice = ForkChoiceView::genesis(HASH32_ZERO, 1, vec![9, 9], vec![8, 8]);
        st.publish(snap);
        let s = handle_route(Route::StatesFinalized, &st, b"");
        assert_eq!(s.status, 200);
        assert_eq!(s.content_type, "application/octet-stream");
        assert_eq!(s.body, vec![9, 9]);
        let b = handle_route(Route::BlocksFinalized, &st, b"");
        assert_eq!(b.content_type, "application/octet-stream");
        assert_eq!(b.body, vec![8, 8]);
    }
}
