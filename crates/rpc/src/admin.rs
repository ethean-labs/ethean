//! Admin control helpers (shutdown request only — no key material).

use crate::auth::{authorize_admin, BindScope};
use crate::error::Result;

/// Request graceful node shutdown via admin API.
pub fn request_shutdown(scope: BindScope, bearer: Option<&str>, token: &str) -> Result<()> {
    authorize_admin(scope, bearer, token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_needs_auth_on_public() {
        assert!(request_shutdown(BindScope::Public, None, "t").is_err());
        assert!(request_shutdown(BindScope::Public, Some("t"), "t").is_ok());
    }
}
