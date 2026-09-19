//! Metrics registry and Lean RPC smoke helpers for the node process.

use ethean_metrics::{
    ensure_core_families, record_slot_gauges, set_ready, MetricsError, Readiness, Registry,
    SharedRegistry,
};
use ethean_rpc::{dispatch, BindScope, IncomingRequest, Route, RpcError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Process-local observability state.
#[derive(Debug)]
pub struct NodeObservability {
    /// Shared metric registry (`ethean_` prefix).
    pub registry: SharedRegistry,
    /// Subsystem readiness bits.
    pub readiness: Readiness,
    /// Atomic mirror of [`Readiness::is_ready`] for the metrics HTTP task.
    pub ready_flag: Arc<AtomicBool>,
}

impl NodeObservability {
    /// Register core families for the given binary version.
    pub fn new(version: &str) -> Result<Self, MetricsError> {
        let mut registry = Registry::with_build_info(version)?;
        ensure_core_families(&mut registry)?;
        set_ready(&mut registry, false)?;
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        let _ = registry.set("start_time_seconds", start);
        Ok(Self {
            registry: SharedRegistry::new(registry),
            readiness: Readiness::default(),
            ready_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Mark storage schema open as healthy.
    pub fn mark_storage_ok(&mut self) {
        self.readiness.storage = true;
    }

    /// Mark crypto surface available (fail-closed backends still count as loaded).
    pub fn mark_crypto_ok(&mut self) {
        self.readiness.crypto = true;
    }

    /// Mark durable signer path available for local smoke.
    pub fn mark_signer_ok(&mut self) {
        self.readiness.signer = true;
    }

    /// Mark UDP/QUIC listen bind attached (swarm dial may still be pending).
    pub fn mark_network_ok(&mut self) {
        self.readiness.network = true;
    }

    /// Mark leanVM prover path available when FFI is wired.
    pub fn mark_prover_ok(&mut self) {
        self.readiness.prover = true;
    }

    /// Apply [`ethean_crypto::FfiStatus`] to crypto/prover readiness bits.
    pub fn apply_ffi_status(&mut self, status: ethean_crypto::FfiStatus) {
        if status.leansig {
            self.mark_crypto_ok();
        }
        // Prover is required only when leanVM is selected; skip the gate otherwise
        // so long-run `/readyz` works for consensus-only builds.
        self.mark_prover_ok();
        let _ = status.leanvm;
    }

    /// Network remains false until QUIC binds; prover false until leanVM FFI.
    pub fn refresh_ready_gauge(&mut self) -> Result<(), MetricsError> {
        let ready = self.readiness.is_ready();
        self.ready_flag.store(ready, Ordering::Relaxed);
        self.registry.with_mut(|reg| set_ready(reg, ready))
    }

    /// Record sync lag and head slot after a duty loop.
    pub fn record_chain(&mut self, head_slot: u64, sync_lag: u64) -> Result<(), MetricsError> {
        self.registry.with_mut(|reg| {
            reg.set("head_slot", head_slot as f64)?;
            reg.set("sync_lag_slots", sync_lag as f64)?;
            Ok(())
        })
    }

    /// Full slot panel snapshot for Grafana (head / justified / finalized / current).
    pub fn record_slots(
        &mut self,
        head_slot: u64,
        justified_slot: u64,
        finalized_slot: u64,
        current_slot: u64,
        sync_lag: u64,
        peer_count: u64,
    ) -> Result<(), MetricsError> {
        self.registry.with_mut(|reg| {
            record_slot_gauges(
                reg,
                head_slot,
                justified_slot,
                finalized_slot,
                current_slot,
                sync_lag,
            )?;
            reg.set("peer_count", peer_count as f64)?;
            Ok(())
        })
    }
}

/// Smoke-dispatch GET `/lean/v1/health` (no Beacon paths).
pub fn smoke_health_route() -> Result<Route, RpcError> {
    let req = IncomingRequest {
        method: "GET",
        path: "/lean/v1/health",
        body_len: 0,
        bearer: None,
    };
    dispatch(&req, BindScope::Loopback, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_route_ok() {
        assert_eq!(smoke_health_route().unwrap(), Route::Health);
    }

    #[test]
    fn ready_without_leanvm_after_ffi() {
        let mut obs = NodeObservability::new("0.1.0").unwrap();
        obs.mark_storage_ok();
        obs.mark_crypto_ok();
        obs.mark_signer_ok();
        obs.mark_network_ok();
        obs.apply_ffi_status(ethean_crypto::FfiStatus::probe());
        obs.refresh_ready_gauge().unwrap();
        assert!(obs.readiness.is_ready());
    }
}
