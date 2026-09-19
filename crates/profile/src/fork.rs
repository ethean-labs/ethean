//! Fork identity placeholder for lstar.
//!
//! Phase 00 left `fork_identifier_bytes` unresolved. This module exposes
//! `fork_name()` only and does **not** invent a 4-byte fork digest.

/// Named fork identity without a digest placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForkId {
    fork_name: &'static str,
}

impl ForkId {
    /// Pinned lstar fork name from Phase 00 / leanSpec.
    pub fn lstar() -> Self {
        Self {
            fork_name: "lstar",
        }
    }

    /// Construct from an explicit non-empty name (validated by callers).
    pub fn from_name(fork_name: &'static str) -> Self {
        Self { fork_name }
    }

    pub fn fork_name(self) -> &'static str {
        self.fork_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lstar_name_only() {
        assert_eq!(ForkId::lstar().fork_name(), "lstar");
    }
}
