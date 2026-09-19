//! Helpers to record common chain/network/validator gauges.

use crate::error::Result;
use crate::registry::{MetricKind, Registry};

/// Ensure standard families exist (idempotent registration attempt).
pub fn ensure_core_families(reg: &mut Registry) -> Result<()> {
    let specs = [
        ("head_slot", "Canonical head slot", MetricKind::Gauge),
        ("finalized_slot", "Finalized checkpoint slot", MetricKind::Gauge),
        ("sync_lag_slots", "Slots behind peer horizon", MetricKind::Gauge),
        ("peer_count", "Connected peers", MetricKind::Gauge),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_families_register_once() {
        let mut r = Registry::with_build_info("0.1.0").unwrap();
        ensure_core_families(&mut r).unwrap();
        ensure_core_families(&mut r).unwrap();
        set_ready(&mut r, true).unwrap();
    }
}
