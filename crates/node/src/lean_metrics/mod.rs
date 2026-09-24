//! Node-side recording of the leanMetrics standard families.
//!
//! Each helper maps one Ethean event onto the `lean_*` series defined by
//! leanEthereum/leanMetrics; see `ethean_metrics::lean` for the table.

mod gossip;
pub mod coverage;
mod peers;

pub use gossip::{
    aggregate_checked, attestation_checked, gossip_aggregation, gossip_attestation, gossip_block,
};
pub use peers::{peer_connect_failed, peer_connected, peer_disconnected};

use gossip::now_ms;

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use ethean_metrics::lean::{inc, observe, set};
use ethean_transition::TransitionTimings;

use crate::chain_owner::ChainOwner;

/// A locally built Type-1 aggregate.
pub fn aggregate_built(coverage: u32, building: Duration) {
    inc("lean_pq_sig_aggregated_signatures_total", &[], 1.0);
    inc(
        "lean_pq_sig_attestations_in_aggregated_signatures_total",
        &[],
        coverage as f64,
    );
    observe(
        "lean_pq_sig_aggregated_signatures_building_time_seconds",
        &[],
        building.as_secs_f64(),
    );
    observe(
        "lean_committee_signatures_aggregation_time_seconds",
        &[],
        building.as_secs_f64(),
    );
}

/// An aggregation cycle that did not submit a job.
pub fn aggregation_skipped(reason: &'static str) {
    inc("lean_aggregator_skipped_total", &[reason], 1.0);
}

/// A state transition applied to a block (imported or own).
pub fn transition(total: Duration, t: &TransitionTimings) {
    observe(
        "lean_state_transition_time_seconds",
        &[],
        total.as_secs_f64(),
    );
    inc(
        "lean_state_transition_slots_processed_total",
        &[],
        t.slots_processed as f64,
    );
    observe(
        "lean_state_transition_slots_processing_time_seconds",
        &[],
        t.slots.as_secs_f64(),
    );
    observe(
        "lean_state_transition_block_processing_time_seconds",
        &[],
        t.block.as_secs_f64(),
    );
    inc(
        "lean_state_transition_attestations_processed_total",
        &[],
        t.attestations_processed as f64,
    );
    observe(
        "lean_state_transition_attestations_processing_time_seconds",
        &[],
        t.attestations.as_secs_f64(),
    );
}

/// Time between accepted duty ticks.
pub fn tick() {
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(previous) = last.replace(Instant::now()) {
        observe(
            "lean_tick_interval_duration_seconds",
            &[],
            previous.elapsed().as_secs_f64(),
        );
    }
}

/// Gauges refreshed from the owner on every metrics refresh.
pub fn refresh(owner: &ChainOwner, current_slot: u64, peers: u64) {
    refresh_with_clients(owner, current_slot, peers, &[], &[]);
}

/// Like [`refresh`], with connected and mesh peers grouped by client family
/// (leanMetrics `client` label); an empty grouping reports `unknown`.
pub fn refresh_with_clients(
    owner: &ChainOwner,
    current_slot: u64,
    peers: u64,
    peer_clients: &[(String, u64)],
    mesh_clients: &[(String, u64)],
) {
    set("lean_current_slot", &[], current_slot as f64);
    set(
        "lean_safe_target_slot",
        &[],
        owner.safe_target_slot() as f64,
    );
    set(
        "lean_fork_choice_reorgs_total",
        &[],
        owner.reorg_total as f64,
    );
    if peer_clients.is_empty() {
        set("lean_connected_peers", &["unknown"], peers as f64);
    } else {
        for (client, n) in peer_clients {
            set("lean_connected_peers", &[client.as_str()], *n as f64);
        }
    }
    if mesh_clients.is_empty() {
        set("lean_gossip_mesh_peers", &["unknown"], 0.0);
    } else {
        for (client, n) in mesh_clients {
            set("lean_gossip_mesh_peers", &[client.as_str()], *n as f64);
        }
    }
    set(
        "lean_gossip_signatures",
        &[],
        owner.signatures.signature_count() as f64,
    );
    set(
        "lean_latest_new_aggregated_payloads",
        &[],
        owner.aggregates.len() as f64,
    );
    set(
        "lean_latest_known_aggregated_payloads",
        &[],
        owner.known_payloads.len() as f64,
    );
    set("lean_is_aggregator", &[], owner.is_aggregator as u8 as f64);
    let synced = if owner.syncing { "syncing" } else { "synced" };
    for status in ["idle", "syncing", "synced"] {
        set(
            "lean_node_sync_status",
            &[status],
            (status == synced) as u8 as f64,
        );
    }
    if let Some(state) = owner.head_state.as_ref() {
        finalization_progress(state.latest_finalized.slot.get());
        set("lean_head_slot", &[], state.slot.get() as f64);
        for name in ["lean_latest_justified_slot", "lean_justified_slot"] {
            set(name, &[], state.latest_justified.slot.get() as f64);
        }
        for name in ["lean_latest_finalized_slot", "lean_finalized_slot"] {
            set(name, &[], state.latest_finalized.slot.get() as f64);
        }
    }
}

/// Count a finalization each time the finalized slot advances.
fn finalization_progress(current: u64) {
    static LAST: Mutex<Option<u64>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|p| p.into_inner());
    if last.is_some_and(|previous| current > previous) {
        inc("lean_finalizations_total", &["success"], 1.0);
    }
    *last = Some(current);
}

/// Static node facts; the start time is captured on the first call.
pub fn node_facts(owner: &ChainOwner, version: &str) {
    static START: OnceLock<u64> = OnceLock::new();
    let start = *START.get_or_init(|| now_ms() / 1000);
    set("lean_node_info", &["ethean", version], 1.0);
    set("lean_node_start_time_seconds", &[], start as f64);
    let registry = owner
        .head_state
        .as_ref()
        .map(|s| s.validators.len())
        .unwrap_or(0);
    let managed = if owner.owned_validator_indices.is_empty() {
        registry
    } else {
        owner.owned_validator_indices.len()
    };
    set("lean_validators_count", &[], managed as f64);
    let committees = owner
        .profile
        .as_ref()
        .map(|p| p.attestation_committee_count.max(1))
        .unwrap_or(1);
    set("lean_attestation_committee_count", &[], committees as f64);
    if let Some(index) = owner.owned_validator_indices.first() {
        set(
            "lean_attestation_committee_subnet",
            &[],
            (index % committees) as f64,
        );
    }
}
