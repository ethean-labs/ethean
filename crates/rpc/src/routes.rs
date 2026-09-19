//! Lean API route table (no /eth/v1 Beacon compatibility).

use crate::error::{Result, RpcError};

/// Known Lean HTTP paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// GET /lean/v1/health
    Health,
    /// GET /lean/v1/ready
    Ready,
    /// GET /lean/v1/node/identity
    NodeIdentity,
    /// GET /lean/v1/chain/head
    ChainHead,
    /// GET /lean/v1/chain/finalized
    ChainFinalized,
    /// GET /lean/v1/chain/sync
    ChainSync,
    /// GET /lean/v1/validator/duties (bounded visibility)
    ValidatorDuties,
    /// POST /lean/v1/admin/shutdown
    AdminShutdown,
    /// GET /lean/v1/events (authenticated stream)
    AdminEvents,
}

/// Parse method + path into a Lean route; reject Beacon paths.
pub fn match_route(method: &str, path: &str) -> Result<Route> {
    if path.starts_with("/eth/") {
        return Err(RpcError::UnknownRoute(path.to_string()));
    }
    match (method, path) {
        ("GET", "/lean/v1/health") => Ok(Route::Health),
        ("GET", "/lean/v1/ready") => Ok(Route::Ready),
        ("GET", "/lean/v1/node/identity") => Ok(Route::NodeIdentity),
        ("GET", "/lean/v1/chain/head") => Ok(Route::ChainHead),
        ("GET", "/lean/v1/chain/finalized") => Ok(Route::ChainFinalized),
        ("GET", "/lean/v1/chain/sync") => Ok(Route::ChainSync),
        ("GET", "/lean/v1/validator/duties") => Ok(Route::ValidatorDuties),
        ("POST", "/lean/v1/admin/shutdown") => Ok(Route::AdminShutdown),
        ("GET", "/lean/v1/events") => Ok(Route::AdminEvents),
        _ => Err(RpcError::UnknownRoute(format!("{method} {path}"))),
    }
}

/// True when the route requires admin authorization.
pub fn requires_admin(route: Route) -> bool {
    matches!(route, Route::AdminShutdown | Route::AdminEvents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_beacon_paths() {
        assert!(match_route("GET", "/eth/v1/node/health").is_err());
    }

    #[test]
    fn matches_lean_health() {
        assert_eq!(match_route("GET", "/lean/v1/health").unwrap(), Route::Health);
    }
}
