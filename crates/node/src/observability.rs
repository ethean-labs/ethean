//! Metrics registry and Lean RPC smoke helpers for the node process.

use ethean_metrics::{
    ensure_core_families, set_ready, MetricsError, Readiness, Registry,
};
use ethean_rpc::{
    dispatch, BindScope, IncomingRequest, Route, RpcError,
};

/// Process-local observability state.
#[derive(Debug)]
pub struct NodeObservability {
    /// Metric registry (`ethean_` prefix).
    pub registry: Registry,
    /// Subsystem readiness bits.
    pub readiness: Readiness,
}

impl NodeObservability {
    /// Register core families for the given binary version.
    pub fn new(version: &str) -> Result<Self, MetricsError> {
        let mut registry = Registry::with_build_info(version)?;
        ensure_core_families(&mut registry)?;
        set_ready(&mut registry, false)?;
        Ok(Self {
            registry,
            readiness: Readiness::default(),
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

    /// Network remains false until QUIC binds; prover false until leanVM FFI.
    pub fn refresh_ready_gauge(&mut self) -> Result<(), MetricsError> {
        set_ready(&mut self.registry, self.readiness.is_ready())
    }

    /// Record sync lag and head slot after a duty loop.
    pub fn record_chain(&mut self, head_slot: u64, sync_lag: u64) -> Result<(), MetricsError> {
        self.registry.set("head_slot", head_slot as f64)?;
        self.registry.set("sync_lag_slots", sync_lag as f64)?;
        Ok(())
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
    fn not_ready_until_all_gates() {
        let mut obs = NodeObservability::new("0.1.0").unwrap();
        obs.mark_storage_ok();
        obs.mark_crypto_ok();
        obs.mark_signer_ok();
        obs.refresh_ready_gauge().unwrap();
        assert!(!obs.readiness.is_ready());
    }
}
