//! Fork-choice store options.

/// Options controlling attestation proof requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForkChoiceOpts {
    /// When true, attestation ingest that would need XMSS/proofs is rejected
    /// with [`crate::ForkChoiceError::UnsupportedSignature`] (no fake accept).
    pub require_proofs: bool,
}

impl Default for ForkChoiceOpts {
    fn default() -> Self {
        Self {
            require_proofs: true,
        }
    }
}

impl ForkChoiceOpts {
    /// Production default: proofs required (deferred → UnsupportedSignature).
    pub const REQUIRE_PROOFS: Self = Self {
        require_proofs: true,
    };

    /// Test / fixture path: accept structural AttestationData without proofs.
    pub const STRUCTURAL: Self = Self {
        require_proofs: false,
    };
}
