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

/// Detailed leanVM gate (safe to call without the `leanvm-backend` feature).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeanVmGate {
    /// Upstream pin string (40-char hex when known).
    pub pinned_rev: &'static str,
    /// Cargo feature `leanvm-backend` compiled in.
    pub feature_enabled: bool,
    /// `LEANVM_FFI_LINKED` is true.
    pub ffi_linked: bool,
}

impl LeanVmGate {
    /// Probe this build.
    pub fn probe() -> Self {
        #[cfg(feature = "leanvm-backend")]
        {
            let s = crate::backend_leanvm::LeanVmLinkStatus::probe();
            Self {
                pinned_rev: s.pinned_rev,
                feature_enabled: s.feature_enabled,
                ffi_linked: s.ffi_linked,
            }
        }
        #[cfg(not(feature = "leanvm-backend"))]
        {
            Self {
                pinned_rev: crate::aggregation::LEANVM_REV,
                feature_enabled: false,
                ffi_linked: false,
            }
        }
    }

    /// Production prove/verify may run only when feature + FFI are both true.
    pub fn ready(self) -> bool {
        self.feature_enabled && self.ffi_linked
    }
}

impl FfiStatus {
    /// Probe compile-time features and link flags (never claims leanVM without FFI).
    pub fn probe() -> Self {
        Self {
            leansig: cfg!(feature = "leansig-backend"),
            leanvm: LeanVmGate::probe().ready(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_build_reports_gaps() {
        let s = FfiStatus::probe();
        assert!(!s.both_selected());
        assert!(!s.gaps().is_empty());
        assert!(!s.leanvm);
        let g = LeanVmGate::probe();
        assert!(!g.ready());
        assert_eq!(g.pinned_rev.len(), 40);
    }

    #[cfg(feature = "leanvm-backend")]
    #[test]
    fn leanvm_feature_alone_is_not_ready() {
        assert!(!crate::backend_leanvm::LEANVM_FFI_LINKED);
        assert!(!FfiStatus::probe().leanvm);
        assert!(LeanVmGate::probe().feature_enabled);
        assert!(!LeanVmGate::probe().ready());
    }
}
