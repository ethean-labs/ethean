//! Metrics registry and Lean RPC smoke helpers for the node process.

use ethean_metrics::{
    ensure_core_families, record_admin_event_backlog, record_bootnode_count,
    record_durable_persist, record_fc_reorg, record_fc_reorg_total, record_range_serve,
    record_readiness_gauges, record_role_gauges, record_serve_cache_seed, record_slot_gauges,
    set_ready, MetricsError, Readiness, Registry, SharedRegistry,
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

    /// Mark the aggregate-proof path available.
    pub fn mark_prover_ok(&mut self) {
        self.readiness.prover = true;
    }

    /// Apply the crypto capability probe to readiness bits. XMSS and proof
    /// verification are always in-process; proving is optional per role.
    pub fn apply_crypto_status(&mut self, status: &crate::crypto_status::CryptoStatus) {
        if status.xmss {
            self.mark_crypto_ok();
        }
        self.mark_prover_ok();
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

    /// Full slot panel snapshot for Grafana (head / justified / finalized / safe / current).
    pub fn record_slots(
        &mut self,
        head_slot: u64,
        justified_slot: u64,
        finalized_slot: u64,
        safe_target_slot: u64,
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
                safe_target_slot,
                current_slot,
                sync_lag,
            )?;
            reg.set("peer_count", peer_count as f64)?;
            Ok(())
        })
    }

    /// Increment when the canonical head moves onto a competing branch.
    pub fn record_reorg(&mut self) -> Result<(), MetricsError> {
        self.registry.with_mut(record_fc_reorg)
    }

    /// Publish the absolute reorg counter from the chain owner.
    pub fn record_reorg_total(&mut self, total: u64) -> Result<(), MetricsError> {
        self.registry
            .with_mut(|reg| record_fc_reorg_total(reg, total))
    }

    /// Validator count, aggregator / local-finality flags, genesis clock.
    pub fn record_roles(
        &mut self,
        validator_count: u64,
        aggregator: bool,
        local_finality: bool,
        genesis_time_seconds: u64,
        seconds_per_slot: u64,
    ) -> Result<(), MetricsError> {
        self.registry.with_mut(|reg| {
            record_role_gauges(
                reg,
                validator_count,
                aggregator,
                local_finality,
                genesis_time_seconds,
                seconds_per_slot,
            )
        })
    }

    /// Mirror readiness subsystem bits into gauges for the ops dashboard.
    pub fn record_readiness_bits(&mut self) -> Result<(), MetricsError> {
        let r = self.readiness;
        self.registry.with_mut(|reg| {
            record_readiness_gauges(reg, r.storage, r.crypto, r.signer, r.network, r.prover)
        })
    }

    /// How many bootnode multiaddrs were configured for this process.
    pub fn record_bootnodes(&mut self, bootnode_count: u64) -> Result<(), MetricsError> {
        self.registry
            .with_mut(|reg| record_bootnode_count(reg, bootnode_count))
    }

    /// Accrue durable flush / prune samples after a successful data-dir save.
    pub fn record_durable_persist(
        &mut self,
        flushed_blocks: u64,
        floor_slot: u64,
        files_removed: u64,
        redb_removed: u64,
        keep_slots: u64,
    ) -> Result<(), MetricsError> {
        self.registry.with_mut(|reg| {
            record_durable_persist(
                reg,
                flushed_blocks,
                floor_slot,
                files_removed,
                redb_removed,
                keep_slots,
            )
        })
    }

    /// Mirror QuicSwarm blocks-by-range serve atomics into Prometheus.
    pub fn record_range_serve(
        &mut self,
        found_total: u64,
        missing_total: u64,
        cache_slots: u64,
    ) -> Result<(), MetricsError> {
        self.registry
            .with_mut(|reg| record_range_serve(reg, found_total, missing_total, cache_slots))
    }

    /// Record durable serve-cache seed counts from boot.
    pub fn record_serve_cache_seed(
        &mut self,
        candidates: u64,
        indexed: u64,
    ) -> Result<(), MetricsError> {
        self.registry
            .with_mut(|reg| record_serve_cache_seed(reg, candidates, indexed))
    }

    /// Publish Lean HTTP admin event ring-buffer depth for scrape / Grafana.
    pub fn record_admin_event_backlog(&mut self, pending: u64) -> Result<(), MetricsError> {
        self.registry
            .with_mut(|reg| record_admin_event_backlog(reg, pending))
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
        obs.apply_crypto_status(&crate::crypto_status::CryptoStatus::probe());
        obs.refresh_ready_gauge().unwrap();
        assert!(obs.readiness.is_ready());
    }
}
