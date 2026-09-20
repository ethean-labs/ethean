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
    /// `LEANVM_FFI_LINKED` is true (in-process symbols).
    pub ffi_linked: bool,
    /// `ETHEAN_LEANVM_PROVER` points at an existing file.
    pub ipc_binary_present: bool,
    /// Framed process IPC protocol is implemented.
    pub ipc_protocol_ready: bool,
}

/// Detailed leanSig gate (compile feature + recorded upstream pin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeanSigGate {
    /// Upstream leanSig git rev pin.
    pub pinned_rev: &'static str,
    /// Cargo feature `leansig-backend` compiled in.
    pub feature_enabled: bool,
}

impl LeanSigGate {
    /// Probe this build.
    pub fn probe() -> Self {
        Self {
            pinned_rev: crate::xmss::LEANSIG_REV,
            feature_enabled: cfg!(feature = "leansig-backend"),
        }
    }

    /// True when the production XMSS backend is compiled in.
    ///
    /// Does not guarantee the git dep resolves without the local num-bigint vendor patch.
    pub fn ready(self) -> bool {
        self.feature_enabled
    }

    /// Human-readable gap when production leanSig cannot serve peer verify.
    pub fn refuse_reason(self) -> Option<&'static str> {
        if self.ready() {
            None
        } else {
            Some("leansig-backend feature disabled; refuse always-true XMSS verify")
        }
    }
}

impl LeanVmGate {
    /// Probe this build.
    pub fn probe() -> Self {
        let ipc = crate::leanvm_ipc::LeanVmIpcStatus::probe();
        #[cfg(feature = "leanvm-backend")]
        {
            let s = crate::backend_leanvm::LeanVmLinkStatus::probe();
            Self {
                pinned_rev: s.pinned_rev,
                feature_enabled: s.feature_enabled,
                ffi_linked: s.ffi_linked,
                ipc_binary_present: ipc.binary_present,
                ipc_protocol_ready: ipc.protocol_ready,
            }
        }
        #[cfg(not(feature = "leanvm-backend"))]
        {
            Self {
                pinned_rev: crate::aggregation::LEANVM_REV,
                feature_enabled: false,
                ffi_linked: false,
                ipc_binary_present: ipc.binary_present,
                ipc_protocol_ready: ipc.protocol_ready,
            }
        }
    }

    /// Production prove/verify may run when in-process FFI or process IPC is ready.
    pub fn ready(self) -> bool {
        self.feature_enabled && (self.ffi_linked || self.ipc_ready())
    }

    /// Process-isolated prover path is fully wired.
    pub fn ipc_ready(self) -> bool {
        self.ipc_binary_present && self.ipc_protocol_ready
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
        let s = LeanSigGate::probe();
        assert_eq!(s.pinned_rev.len(), 40);
        assert!(!s.ready());
        assert!(s.refuse_reason().is_some());
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
