//! Gossip sizes, arrival timing, and vote / aggregate validation outcomes.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ethean_metrics::lean::{inc, observe};

use crate::chain_owner::ChainOwner;

/// Interval (within a slot) each gossip kind is due in (leanSpec lstar).
const BLOCK_INTERVAL: u64 = 0;
const ATTESTATION_INTERVAL: u64 = 1;
const AGGREGATION_INTERVAL: u64 = 2;

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// `(genesis_ms, ms_per_slot, ms_per_interval)` when the owner knows them.
fn timing(owner: &ChainOwner) -> Option<(u64, u64, u64)> {
    let genesis_ms = owner
        .head_state
        .as_ref()?
        .config
        .genesis_time
        .saturating_mul(1000);
    let profile = owner.profile.as_ref()?;
    let per_slot = profile.seconds_per_slot.saturating_mul(1000);
    let per_interval = per_slot / profile.intervals_per_slot.max(1);
    Some((genesis_ms, per_slot, per_interval))
}

fn position(delta_ms: i64, per_interval: u64) -> &'static str {
    if delta_ms < 0 {
        "before"
    } else if delta_ms < per_interval as i64 {
        "inside"
    } else {
        "after"
    }
}

fn record_arrival(delay: &str, total: &str, delta_ms: i64, per_interval: u64) {
    observe(delay, &[], delta_ms.unsigned_abs() as f64 / 1000.0);
    inc(total, &[position(delta_ms, per_interval)], 1.0);
}

/// Arrival of a block or vote against its own slot's due interval. Slots
/// more than one slot ahead of the wall clock are attacker-chosen and skipped.
fn due_arrival(owner: &ChainOwner, slot: u64, interval: u64, delay: &str, total: &str) {
    let Some((genesis, per_slot, per_interval)) = timing(owner) else {
        return;
    };
    let now = now_ms();
    let due = genesis + slot.saturating_mul(per_slot) + interval * per_interval;
    if due > now + per_slot {
        return;
    }
    record_arrival(delay, total, now as i64 - due as i64, per_interval);
}

pub fn gossip_block(owner: &ChainOwner, slot: u64, bytes: usize) {
    observe("lean_gossip_block_size_bytes", &[], bytes as f64);
    due_arrival(
        owner,
        slot,
        BLOCK_INTERVAL,
        "lean_gossip_block_arrival_delay_seconds",
        "lean_gossip_block_arrival_total",
    );
}

pub fn gossip_attestation(owner: &ChainOwner, slot: u64, bytes: usize) {
    observe("lean_gossip_attestation_size_bytes", &[], bytes as f64);
    due_arrival(
        owner,
        slot,
        ATTESTATION_INTERVAL,
        "lean_gossip_attestation_arrival_delay_seconds",
        "lean_gossip_attestation_arrival_total",
    );
}

/// Aggregates are measured against the latest aggregation boundary, not their
/// data slot (catch-up aggregates may carry older slots).
pub fn gossip_aggregation(owner: &ChainOwner, bytes: usize) {
    observe("lean_gossip_aggregation_size_bytes", &[], bytes as f64);
    let Some((genesis, per_slot, per_interval)) = timing(owner) else {
        return;
    };
    let since = now_ms().saturating_sub(genesis) as i64;
    let delta = (since - (AGGREGATION_INTERVAL * per_interval) as i64).rem_euclid(per_slot as i64);
    let delay = "lean_gossip_aggregation_arrival_delay_seconds";
    observe(delay, &[], delta as f64 / 1000.0);
    let pos = if delta < per_interval as i64 {
        "inside"
    } else {
        "after"
    };
    inc("lean_gossip_aggregation_arrival_total", &[pos], 1.0);
}

/// One validated (or rejected) individual vote and its signature check.
pub fn attestation_checked(
    valid: bool,
    signature_checked: bool,
    validation: Duration,
    verification: Duration,
) {
    let v = if valid {
        "lean_attestations_valid_total"
    } else {
        "lean_attestations_invalid_total"
    };
    inc(v, &[], 1.0);
    observe(
        "lean_attestation_validation_time_seconds",
        &[],
        validation.as_secs_f64(),
    );
    if signature_checked {
        let s = if valid {
            "lean_pq_sig_attestation_signatures_valid_total"
        } else {
            "lean_pq_sig_attestation_signatures_invalid_total"
        };
        inc(s, &[], 1.0);
        observe(
            "lean_pq_sig_attestation_verification_time_seconds",
            &[],
            verification.as_secs_f64(),
        );
    }
}

/// One validated (or rejected) aggregate and its proof check.
pub fn aggregate_checked(
    valid: bool,
    proof_checked: bool,
    validation: Duration,
    verification: Duration,
) {
    let v = if valid {
        "lean_attestations_valid_total"
    } else {
        "lean_attestations_invalid_total"
    };
    inc(v, &[], 1.0);
    observe(
        "lean_attestation_validation_time_seconds",
        &[],
        validation.as_secs_f64(),
    );
    if proof_checked {
        let s = if valid {
            "lean_pq_sig_aggregated_signatures_valid_total"
        } else {
            "lean_pq_sig_aggregated_signatures_invalid_total"
        };
        inc(s, &[], 1.0);
        observe(
            "lean_pq_sig_aggregated_signatures_verification_time_seconds",
            &[],
            verification.as_secs_f64(),
        );
    }
}
