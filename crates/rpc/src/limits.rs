//! RPC body and rate budgets.

/// Maximum JSON request body bytes.
pub const MAX_BODY_BYTES: usize = 256 * 1024;

/// Maximum admin event backlog.
pub const MAX_EVENT_BACKLOG: usize = 128;

/// Public requests per peer token per second (soft policy).
pub const PUBLIC_RPS: u32 = 32;

/// Admin requests per authenticated principal per second.
pub const ADMIN_RPS: u32 = 8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_cap_is_256kib() {
        assert_eq!(MAX_BODY_BYTES, 256 * 1024);
    }
}
