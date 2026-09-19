//! Admin authentication and bind policy.

use crate::error::{Result, RpcError};

/// Whether the listener is loopback-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindScope {
    /// 127.0.0.1 / ::1 only — admin may omit token in local ops.
    Loopback,
    /// Non-loopback — admin requires a bearer token.
    Public,
}

/// Check admin access.
pub fn authorize_admin(scope: BindScope, bearer: Option<&str>, expected_token: &str) -> Result<()> {
    match scope {
        BindScope::Loopback => Ok(()),
        BindScope::Public => match bearer {
            Some(t) if t == expected_token && !expected_token.is_empty() => Ok(()),
            Some(_) => Err(RpcError::Unauthorized),
            None => Err(RpcError::Forbidden),
        },
    }
}

/// Reject empty production tokens on public binds at startup.
pub fn validate_admin_token(scope: BindScope, token: &str) -> Result<()> {
    if scope == BindScope::Public && token.is_empty() {
        return Err(RpcError::Forbidden);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_requires_token() {
        assert!(authorize_admin(BindScope::Public, None, "secret").is_err());
        assert!(authorize_admin(BindScope::Public, Some("secret"), "secret").is_ok());
    }

    #[test]
    fn loopback_ok_without_token() {
        assert!(authorize_admin(BindScope::Loopback, None, "").is_ok());
    }
}
