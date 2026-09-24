//! Lean API route table (`/lean/v0` hive surface; `/lean/v1` aliases; no `/eth/`).

use crate::error::{Result, RpcError};

/// Known Lean HTTP paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// GET `/lean/v0/health` (and `/lean/v1/health`).
    Health,
    /// GET `/lean/v0/ready` / `/lean/v1/ready`.
    Ready,
    /// GET `/lean/v0/node/identity` / `/lean/v1/node/identity`.
    NodeIdentity,
    /// GET `/lean/v1/chain/head`.
    ChainHead,
    /// GET `/lean/v1/chain/finalized`.
    ChainFinalized,
    /// GET `/lean/v1/chain/sync`.
    ChainSync,
    /// GET `/lean/v1/chain/fork_choice` (live-store stats).
    ChainForkChoice,
    /// GET `/lean/v1/validator/duties` (bounded visibility).
    ValidatorDuties,
    /// POST `/lean/v1/admin/shutdown`.
    AdminShutdown,
    /// GET `/lean/v0/events` / `/lean/v1/events`.
    AdminEvents,
    /// GET `/lean/v0/checkpoints/justified`.
    CheckpointsJustified,
    /// GET `/lean/v0/fork_choice`.
    ForkChoice,
    /// GET `/lean/v0/states/finalized`.
    StatesFinalized,
    /// GET `/lean/v0/blocks/finalized`.
    BlocksFinalized,
    /// GET `/lean/v0/admin/aggregator`.
    AggregatorGet,
    /// POST `/lean/v0/admin/aggregator`.
    AggregatorPost,
    /// GET `/metrics` (optional 5052 proxy of the 9100 scrape).
    Metrics,
    /// POST `/lean/v0/test_driver/fork_choice/init` (hive spec assets).
    DriverForkChoiceInit,
    /// POST `/lean/v0/test_driver/fork_choice/step`.
    DriverForkChoiceStep,
    /// POST `/lean/v0/test_driver/state_transition/run`.
    DriverStateTransition,
    /// POST `/lean/v0/test_driver/verify_signatures/run`.
    DriverVerifySignatures,
}

/// Parse method + path into a Lean route; reject Beacon paths.
pub fn match_route(method: &str, path: &str) -> Result<Route> {
    if path.starts_with("/eth/") {
        return Err(RpcError::UnknownRoute(path.to_string()));
    }
    let path = strip_trailing_slash(path);
    match (method, path) {
        ("GET", "/lean/v0/health" | "/lean/v1/health") => Ok(Route::Health),
        ("GET", "/lean/v0/ready" | "/lean/v1/ready") => Ok(Route::Ready),
        ("GET", "/lean/v0/node/identity" | "/lean/v1/node/identity") => Ok(Route::NodeIdentity),
        ("GET", "/lean/v1/chain/head") => Ok(Route::ChainHead),
        ("GET", "/lean/v1/chain/finalized") => Ok(Route::ChainFinalized),
        ("GET", "/lean/v1/chain/sync") => Ok(Route::ChainSync),
        ("GET", "/lean/v1/chain/fork_choice") => Ok(Route::ChainForkChoice),
        ("GET", "/lean/v1/validator/duties") => Ok(Route::ValidatorDuties),
        ("POST", "/lean/v1/admin/shutdown" | "/lean/v0/admin/shutdown") => Ok(Route::AdminShutdown),
        ("GET", "/lean/v0/events" | "/lean/v1/events") => Ok(Route::AdminEvents),
        ("GET", "/lean/v0/checkpoints/justified" | "/lean/v1/checkpoints/justified") => {
            Ok(Route::CheckpointsJustified)
        }
        ("GET", "/lean/v0/fork_choice" | "/lean/v1/fork_choice") => Ok(Route::ForkChoice),
        ("GET", "/lean/v0/states/finalized" | "/lean/v1/states/finalized") => {
            Ok(Route::StatesFinalized)
        }
        ("GET", "/lean/v0/blocks/finalized" | "/lean/v1/blocks/finalized") => {
            Ok(Route::BlocksFinalized)
        }
        ("GET", "/lean/v0/admin/aggregator" | "/lean/v1/admin/aggregator") => {
            Ok(Route::AggregatorGet)
        }
        ("POST", "/lean/v0/admin/aggregator" | "/lean/v1/admin/aggregator") => {
            Ok(Route::AggregatorPost)
        }
        ("GET", "/metrics") => Ok(Route::Metrics),
        ("POST", "/lean/v0/test_driver/fork_choice/init") => Ok(Route::DriverForkChoiceInit),
        ("POST", "/lean/v0/test_driver/fork_choice/step") => Ok(Route::DriverForkChoiceStep),
        ("POST", "/lean/v0/test_driver/state_transition/run") => {
            Ok(Route::DriverStateTransition)
        }
        ("POST", "/lean/v0/test_driver/verify_signatures/run") => {
            Ok(Route::DriverVerifySignatures)
        }
        _ => Err(RpcError::UnknownRoute(format!("{method} {path}"))),
    }
}

fn strip_trailing_slash(path: &str) -> &str {
    if path.len() > 1 {
        path.strip_suffix('/').unwrap_or(path)
    } else {
        path
    }
}

/// True when the route requires admin authorization.
pub fn requires_admin(route: Route) -> bool {
    matches!(
        route,
        Route::AdminShutdown | Route::AdminEvents | Route::AggregatorPost
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_beacon_paths() {
        assert!(match_route("GET", "/eth/v1/node/health").is_err());
    }

    #[test]
    fn matches_v0_and_v1_health() {
        assert_eq!(match_route("GET", "/lean/v1/health").unwrap(), Route::Health);
        assert_eq!(match_route("GET", "/lean/v0/health").unwrap(), Route::Health);
        assert_eq!(match_route("GET", "/lean/v0/ready").unwrap(), Route::Ready);
        assert_eq!(match_route("GET", "/lean/v1/ready").unwrap(), Route::Ready);
        assert_eq!(
            match_route("GET", "/lean/v0/node/identity").unwrap(),
            Route::NodeIdentity
        );
    }

    #[test]
    fn matches_hive_v0_table() {
        assert_eq!(
            match_route("GET", "/lean/v0/checkpoints/justified").unwrap(),
            Route::CheckpointsJustified
        );
        assert_eq!(
            match_route("GET", "/lean/v0/fork_choice").unwrap(),
            Route::ForkChoice
        );
        assert_eq!(
            match_route("GET", "/lean/v0/states/finalized").unwrap(),
            Route::StatesFinalized
        );
        assert_eq!(
            match_route("GET", "/lean/v0/blocks/finalized").unwrap(),
            Route::BlocksFinalized
        );
        assert_eq!(
            match_route("GET", "/lean/v0/admin/aggregator").unwrap(),
            Route::AggregatorGet
        );
        assert_eq!(
            match_route("POST", "/lean/v0/admin/aggregator").unwrap(),
            Route::AggregatorPost
        );
        assert_eq!(
            match_route("GET", "/lean/v0/events").unwrap(),
            Route::AdminEvents
        );
        assert_eq!(
            match_route("GET", "/lean/v1/events").unwrap(),
            Route::AdminEvents
        );
    }

    #[test]
    fn matches_fork_choice() {
        assert_eq!(
            match_route("GET", "/lean/v1/chain/fork_choice").unwrap(),
            Route::ChainForkChoice
        );
    }
}
