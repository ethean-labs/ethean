//! Production crypto backend readiness (leanSig / leanVM pins).

/// Why a production backend is unavailable at compile or link time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendGap {
    /// leanSig PROD crate / Plonky3 stack not linked (`leansig-backend` off or broken).
    LeanSigUnwired,
    /// leanVM FFI not linked (`leanvm-backend` off or stub).
    LeanVmUnwired,
}

/// Snapshot of which production backends can serve verify/sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FfiStatus {
    /// leanSig XMSS PROD path.
    pub leansig: bool,
    /// leanVM aggregate prove/verify path (true only when FFI symbols are linked).
    pub leanvm: bool,
}

impl FfiStatus {
    /// Probe compile-time features and link flags (never claims leanVM without FFI).
    pub fn probe() -> Self {
        Self {
            leansig: cfg!(feature = "leansig-backend"),
            leanvm: leanvm_ready(),
        }
    }

    /// True only when both production backends can serve requests.
    pub fn both_selected(&self) -> bool {
        self.leansig && self.leanvm
    }

    /// Gaps still blocking a full production crypto stack.
    pub fn gaps(&self) -> Vec<BackendGap> {
        let mut out = Vec::new();
        if !self.leansig {
            out.push(BackendGap::LeanSigUnwired);
        }
        if !self.leanvm {
            out.push(BackendGap::LeanVmUnwired);
        }
        out
    }
}

fn leanvm_ready() -> bool {
    #[cfg(feature = "leanvm-backend")]
    {
        crate::backend_leanvm::LEANVM_FFI_LINKED
    }
    #[cfg(not(feature = "leanvm-backend"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_build_reports_gaps() {
        let s = FfiStatus::probe();
        assert!(!s.both_selected());
        assert!(!s.gaps().is_empty());
        assert!(!s.leanvm);
    }

    #[cfg(feature = "leanvm-backend")]
    #[test]
    fn leanvm_feature_alone_is_not_ready() {
        assert!(!crate::backend_leanvm::LEANVM_FFI_LINKED);
        assert!(!FfiStatus::probe().leanvm);
    }
}
