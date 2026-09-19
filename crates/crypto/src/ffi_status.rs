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
    /// leanVM aggregate prove/verify path.
    pub leanvm: bool,
}

impl FfiStatus {
    /// Probe compile-time features (never claims ready without a wired backend).
    pub fn probe() -> Self {
        Self {
            leansig: cfg!(feature = "leansig-backend"),
            leanvm: cfg!(feature = "leanvm-backend"),
        }
    }

    /// True only when both production backends are feature-selected.
    ///
    /// Even then, runtime may still fail closed if FFI symbols are missing;
    /// callers must treat verify/sign errors as authoritative.
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
        // Default workspace features keep production backends off.
        assert!(!s.both_selected());
        assert!(!s.gaps().is_empty());
    }
}
