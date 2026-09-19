//! Helpers to record common chain/network/validator gauges.

use crate::error::Result;
use crate::registry::{MetricKind, Registry};

/// Ensure standard families exist (idempotent registration attempt).
pub fn ensure_core_families(reg: &mut Registry) -> Result<()> {
    let specs = [
        ("head_slot", "Canonical head slot", MetricKind::Gauge),
        ("justified_slot", "Latest justified checkpoint slot", MetricKind::Gauge),
        ("finalized_slot", "Finalized checkpoint slot", MetricKind::Gauge),
        ("slot_current", "Wall-clock current slot", MetricKind::Gauge),
        ("sync_lag_slots", "Slots behind peer horizon", MetricKind::Gauge),
        ("peer_count", "Connected peers", MetricKind::Gauge),
        ("start_time_seconds", "Unix time when this process started", MetricKind::Gauge),
        ("duty_suppressed_total", "Duties suppressed", MetricKind::Counter),
        ("prover_timeout_total", "Prover wall-time timeouts", MetricKind::Counter),
        ("ready", "1 if process ready", MetricKind::Gauge),
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

/// Snapshot chain slot gauges for Prometheus scrape (Shariq-style long-run panels).
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
    }
}
