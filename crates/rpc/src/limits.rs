//! RPC body and rate budgets.

/// Maximum JSON / SSZ request body bytes (hive test-driver payloads).
pub const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

/// Maximum admin event backlog.
pub const MAX_EVENT_BACKLOG: usize = 128;

/// Public requests per peer token per second (soft policy).
pub const PUBLIC_RPS: u32 = 32;

/// Admin requests per authenticated principal per second.
pub const ADMIN_RPS: u32 = 8;

/// Maximum HTTP header section (request line + headers) before the body.
pub const MAX_HEADER_BYTES: usize = 64 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_cap_is_64mib() {
        assert_eq!(MAX_BODY_BYTES, 64 * 1024 * 1024);
    }
}
