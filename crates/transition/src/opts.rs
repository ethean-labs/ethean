//! Caller-controlled transition options.

/// Options for [`crate::transition_block`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionOpts {
    /// When true, require a non-empty aggregate proof and XMSS verification.
    /// Until Phase 07/08, any verified path returns [`crate::TransitionError::UnsupportedSignature`].
    pub require_proofs: bool,
}

impl TransitionOpts {
    /// Structural / fixture path (no signature verification).
    pub const UNVERIFIED: Self = Self {
        require_proofs: false,
    };

    /// Verified API — rejects empty proofs and does not fake-accept.
    pub const REQUIRE_PROOFS: Self = Self {
        require_proofs: true,
    };
}

impl Default for TransitionOpts {
    fn default() -> Self {
        Self::UNVERIFIED
    }
}
