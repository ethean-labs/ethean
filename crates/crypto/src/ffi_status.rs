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
    /// Versioned IPC frame codec is compiled in.
    pub ipc_frame_abi_ready: bool,
    /// Length-prefixed spawn exchange is compiled in.
    pub ipc_spawn_wired: bool,
    /// Framed process IPC spawn/round-trip is implemented.
    pub ipc_protocol_ready: bool,
}

/// Upstream leanSig still pins `num-bigint` 0.4 while Plonky3 pulls 0.5.
///
/// Enabling `leansig-backend` against the git dep alone fails to unify `BigUint`.
/// Operators must run `tools/release/vendor-leansig-bigint-fix.ps1` (or
/// `check-leansig-backend.ps1`) until upstream bumps the pin.
pub const LEANSIG_VENDOR_BIGINT_PATCH_REQUIRED: bool = true;

/// Detailed leanSig gate (compile feature + recorded upstream pin).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeanSigGate {
    /// Upstream leanSig git rev pin.
    pub pinned_rev: &'static str,
    /// Cargo feature `leansig-backend` compiled in.
    pub feature_enabled: bool,
    /// True until upstream leanSig ships `num-bigint` 0.5 with Plonky3.
    pub vendor_bigint_patch_required: bool,
}

impl LeanSigGate {
    /// Probe this build.
    pub fn probe() -> Self {
        Self {
            pinned_rev: crate::xmss::LEANSIG_REV,
            feature_enabled: cfg!(feature = "leansig-backend"),
            vendor_bigint_patch_required: LEANSIG_VENDOR_BIGINT_PATCH_REQUIRED,
        }
    }

    /// True when the production XMSS backend is compiled into this binary.
    ///
    /// Compile success still requires the local num-bigint vendor patch (or an
    /// equivalent `[patch]`) until [`LEANSIG_VENDOR_BIGINT_PATCH_REQUIRED`] is false.
    pub fn ready(self) -> bool {
        self.feature_enabled
    }

    /// Human-readable gap when production leanSig cannot serve peer verify.
    pub fn refuse_reason(self) -> Option<&'static str> {
        if self.ready() {
            None
        } else if self.vendor_bigint_patch_required {
            Some(
                "leansig-backend feature disabled (git dep needs local num-bigint 0.5 vendor patch); refuse always-true XMSS verify",
            )
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
                ipc_frame_abi_ready: ipc.frame_abi_ready,
                ipc_spawn_wired: ipc.spawn_exchange_wired,
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
                ipc_frame_abi_ready: ipc.frame_abi_ready,
                ipc_spawn_wired: ipc.spawn_exchange_wired,
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

    /// Human-readable gap when production leanVM cannot serve Type-1/Type-2 proofs.
    pub fn refuse_reason(self) -> Option<&'static str> {
        if self.ready() {
            return None;
        }
        if !self.feature_enabled {
            return Some("leanvm-backend feature disabled; refuse always-true aggregate verify");
        }
        if self.ipc_binary_present && self.ipc_spawn_wired && !self.ipc_protocol_ready {
            return Some(
                "leanVM IPC spawn wired but pin-checked round-trip not green; refuse ready claim",
            );
        }
        if self.ipc_binary_present && !self.ipc_protocol_ready {
            return Some("leanVM IPC binary present but framed protocol not ready");
        }
        if !self.ffi_linked && !self.ipc_ready() {
            return Some("leanVM FFI unlinked and IPC prover not ready");
        }
        Some("leanVM production backend unavailable")
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
        let sig = LeanSigGate::probe();
        assert_eq!(sig.pinned_rev.len(), 40);
        assert!(sig.vendor_bigint_patch_required);
        #[cfg(not(feature = "leansig-backend"))]
        {
            assert!(!sig.ready());
            assert!(sig.refuse_reason().is_some());
        }
        #[cfg(feature = "leansig-backend")]
        {
            assert!(sig.ready());
            assert!(sig.refuse_reason().is_none());
        }
        assert!(g.refuse_reason().is_some());
    }

    #[cfg(feature = "leansig-backend")]
    #[test]
    fn leansig_feature_reports_ready() {
        let g = LeanSigGate::probe();
        assert!(g.feature_enabled);
        assert!(g.ready());
        assert!(g.vendor_bigint_patch_required);
        assert!(FfiStatus::probe().leansig);
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
