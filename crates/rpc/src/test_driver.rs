//! Hive lean simulator `test_driver` surface (spec-asset suites).
//!
//! The node installs a [`TestDriver`] when `HIVE_LEAN_TEST_DRIVER=1`; without
//! one the routes answer 404. Bodies are the raw fixture JSON the simulator
//! posts (`hive/simulators/lean/src/scenarios/spec_assets.rs`).

use std::sync::Arc;

use crate::error::RpcError;
use crate::handlers::{error_json, HttpReply};
use crate::routes::Route;
use crate::state::SharedApiState;

/// Fixture-driven fork choice, state transition and signature checks.
pub trait TestDriver: Send + Sync {
    /// Replace the store from `{"anchorState","anchorBlock","genesisTime"}`.
    fn fork_choice_init(&self, body: &[u8]) -> Result<(), String>;
    /// Apply one fixture step; returns the `{accepted,error,snapshot}` JSON.
    fn fork_choice_step(&self, body: &[u8]) -> Result<String, String>;
    /// Run a whole state-transition case; returns `{succeeded,error,post}`.
    fn state_transition(&self, body: &[u8]) -> Result<String, String>;
    /// Verify a fixture signed block; returns `{succeeded,error}`.
    fn verify_signatures(&self, body: &[u8]) -> Result<String, String>;
}

/// Serve a driver route against the installed driver.
pub fn handle_driver(route: Route, state: &SharedApiState, body: &[u8]) -> HttpReply {
    let Some(driver) = state.driver() else {
        return error_json(RpcError::UnknownRoute(
            "test driver not enabled (HIVE_LEAN_TEST_DRIVER=1)".into(),
        ));
    };
    match route {
        Route::DriverForkChoiceInit => match driver.fork_choice_init(body) {
            Ok(()) => HttpReply {
                status: 204,
                content_type: "application/json",
                body: Vec::new(),
            },
            Err(e) => error_json(RpcError::BadRequest(e)),
        },
        Route::DriverForkChoiceStep => json_or_400(driver.fork_choice_step(body)),
        Route::DriverStateTransition => json_or_400(driver.state_transition(body)),
        Route::DriverVerifySignatures => json_or_400(driver.verify_signatures(body)),
        _ => error_json(RpcError::UnknownRoute("not a driver route".into())),
    }
}

fn json_or_400(result: Result<String, String>) -> HttpReply {
    match result {
        Ok(json) => HttpReply::json(200, json),
        Err(e) => error_json(RpcError::BadRequest(e)),
    }
}

/// Shared handle type stored on [`SharedApiState`].
pub type DriverHandle = Arc<dyn TestDriver>;
