//! Request dispatch helpers (HTTP server wiring deferred to binary).

use crate::error::{Result, RpcError};
use crate::limits::MAX_BODY_BYTES;
use crate::routes::{match_route, requires_admin, Route};
use crate::auth::{authorize_admin, BindScope};

/// Inbound request after HTTP parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub body_len: usize,
    pub bearer: Option<&'a str>,
}

/// Validate body size and route; enforce admin auth when required.
pub fn dispatch(
    req: &IncomingRequest<'_>,
    scope: BindScope,
    admin_token: &str,
) -> Result<Route> {
    if req.body_len > MAX_BODY_BYTES {
        return Err(RpcError::BodyTooLarge {
            got: req.body_len,
            max: MAX_BODY_BYTES,
        });
    }
    let route = match_route(req.method, req.path)?;
    if requires_admin(route) {
        authorize_admin(scope, req.bearer, admin_token)?;
    }
    Ok(route)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversize_body() {
        let req = IncomingRequest {
            method: "GET",
            path: "/lean/v1/health",
            body_len: MAX_BODY_BYTES + 1,
            bearer: None,
        };
        assert!(dispatch(&req, BindScope::Loopback, "").is_err());
    }
}
