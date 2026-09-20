//! Helpers to record common chain/network/validator gauges.

use crate::error::Result;
use crate::registry::{MetricKind, Registry};

/// Ensure standard families exist (idempotent registration attempt).
pub fn ensure_core_families(reg: &mut Registry) -> Result<()> {
    let specs = [
        ("head_slot", "Canonical head slot", MetricKind::Gauge),
        (
            "justified_slot",
            "Latest justified checkpoint slot",
            MetricKind::Gauge,
        ),
        (
            "finalized_slot",
            "Finalized checkpoint slot",
            MetricKind::Gauge,
        ),
        ("slot_current", "Wall-clock current slot", MetricKind::Gauge),
        (
            "sync_lag_slots",
            "Slots behind peer horizon",
            MetricKind::Gauge,
        ),
        (
            "finality_lag_slots",
            "Head minus finalized slot",
            MetricKind::Gauge,
        ),
        (
            "justification_lag_slots",
            "Head minus justified slot",
            MetricKind::Gauge,
        ),
        ("peer_count", "Connected peers", MetricKind::Gauge),
        (
            "bootnode_count",
            "Configured bootnode multiaddrs for this start",
            MetricKind::Gauge,
        ),
        (
            "validator_count",
            "Validators in local genesis / head state",
            MetricKind::Gauge,
        ),
        (
            "aggregator_enabled",
            "1 if local aggregator role is on",
            MetricKind::Gauge,
        ),
        (
            "local_finality_enabled",
            "1 if solo local-finality path is on",
            MetricKind::Gauge,
        ),
        (
            "genesis_time_seconds",
            "Genesis unix time from local state",
            MetricKind::Gauge,
        ),
        (
            "seconds_per_slot",
            "Configured seconds per slot",
            MetricKind::Gauge,
        ),
        (
            "start_time_seconds",
            "Unix time when this process started",
            MetricKind::Gauge,
        ),
        (
            "duty_suppressed_total",
            "Duties suppressed",
            MetricKind::Counter,
        ),
        (
            "prover_timeout_total",
            "Prover wall-time timeouts",
            MetricKind::Counter,
        ),
        ("ready", "1 if process ready", MetricKind::Gauge),
        ("ready_storage", "1 if storage gate passed", MetricKind::Gauge),
        ("ready_crypto", "1 if crypto gate passed", MetricKind::Gauge),
        ("ready_signer", "1 if signer gate passed", MetricKind::Gauge),
        ("ready_network", "1 if network gate passed", MetricKind::Gauge),
        ("ready_prover", "1 if prover gate passed", MetricKind::Gauge),
    ];
    for (name, help, kind) in specs {
        if reg.set(name, 0.0).is_err() {
            reg.register(name, kind, help, 0.0, vec![])?;
        }
    }
    Ok(())
}

/// Update ready gauge from a boolean.
pub fn set_ready(reg: &mut Registry, ready: bool) -> Result<()> {
    reg.set("ready", if ready { 1.0 } else { 0.0 })
}

/// Snapshot chain slot gauges for Prometheus scrape.
pub fn record_slot_gauges(
    reg: &mut Registry,
    head_slot: u64,
    justified_slot: u64,
    finalized_slot: u64,
    current_slot: u64,
    sync_lag: u64,
) -> Result<()> {
    reg.set("head_slot", head_slot as f64)?;
    reg.set("justified_slot", justified_slot as f64)?;
    reg.set("finalized_slot", finalized_slot as f64)?;
    reg.set("slot_current", current_slot as f64)?;
    reg.set("sync_lag_slots", sync_lag as f64)?;
    reg.set(
        "finality_lag_slots",
        head_slot.saturating_sub(finalized_slot) as f64,
    )?;
    reg.set(
        "justification_lag_slots",
        head_slot.saturating_sub(justified_slot) as f64,
    )?;
    Ok(())
}

/// Local registry size and role flags for Grafana overview panels.
pub fn record_role_gauges(
    reg: &mut Registry,
    validator_count: u64,
    aggregator: bool,
    local_finality: bool,
    genesis_time_seconds: u64,
    seconds_per_slot: u64,
) -> Result<()> {
    reg.set("validator_count", validator_count as f64)?;
    reg.set("aggregator_enabled", if aggregator { 1.0 } else { 0.0 })?;
    reg.set(
        "local_finality_enabled",
        if local_finality { 1.0 } else { 0.0 },
    )?;
    reg.set("genesis_time_seconds", genesis_time_seconds as f64)?;
    reg.set("seconds_per_slot", seconds_per_slot as f64)?;
    Ok(())
}

/// Configured bootnodes (expected mesh size) for the Peers panel context.
pub fn record_bootnode_count(reg: &mut Registry, bootnode_count: u64) -> Result<()> {
    reg.set("bootnode_count", bootnode_count as f64)
}

/// Mirror readiness bits as separate gauges (dashboard “node health” row).
pub fn record_readiness_gauges(
    reg: &mut Registry,
    storage: bool,
    crypto: bool,
    signer: bool,
    network: bool,
    prover: bool,
) -> Result<()> {
    reg.set("ready_storage", if storage { 1.0 } else { 0.0 })?;
    reg.set("ready_crypto", if crypto { 1.0 } else { 0.0 })?;
    reg.set("ready_signer", if signer { 1.0 } else { 0.0 })?;
    reg.set("ready_network", if network { 1.0 } else { 0.0 })?;
    reg.set("ready_prover", if prover { 1.0 } else { 0.0 })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_families_register_once() {
        let mut r = Registry::with_build_info("0.1.0").unwrap();
        ensure_core_families(&mut r).unwrap();
        ensure_core_families(&mut r).unwrap();
        set_ready(&mut r, true).unwrap();
        record_slot_gauges(&mut r, 10, 8, 6, 11, 1).unwrap();
        record_role_gauges(&mut r, 4, true, true, 1_700_000_000, 4).unwrap();
        record_readiness_gauges(&mut r, true, true, true, true, true).unwrap();
    }
}
