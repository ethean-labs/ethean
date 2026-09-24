//! The leanMetrics standard metric set (leanEthereum/leanMetrics `metrics.md`,
//! commit 69f9722). Names, types, buckets and label keys are fixed by that
//! document; `testdata/leanmetrics-69f9722.md` pins it and a test checks this
//! table against it.

/// Prometheus metric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeanKind {
    Counter,
    Gauge,
    Histogram,
}

/// One standard metric.
#[derive(Debug, Clone, Copy)]
pub struct LeanSpec {
    pub name: &'static str,
    pub kind: LeanKind,
    pub help: &'static str,
    pub labels: &'static [&'static str],
    pub buckets: &'static [f64],
}

use LeanKind::{Counter as C, Gauge as G, Histogram as H};

const fn m(
    name: &'static str,
    kind: LeanKind,
    help: &'static str,
    labels: &'static [&'static str],
    buckets: &'static [f64],
) -> LeanSpec {
    LeanSpec {
        name,
        kind,
        help,
        labels,
        buckets,
    }
}

const NONE: &[f64] = &[];
const FAST: &[f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 1.0];
const AGG: &[f64] = &[0.1, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 4.0];
const ARRIVAL: &[f64] = &[0.05, 0.1, 0.2, 0.4, 0.8, 1.2, 1.6, 2.4, 4.0, 8.0, 16.0];

/// Every metric in the standard, in document order.
pub const LEAN_METRICS: &[LeanSpec] = &[
    m("lean_node_info", G, "Node information (always 1)", &["name", "version"], NONE),
    m("lean_node_start_time_seconds", G, "Start timestamp", &[], NONE),
    m("lean_pq_sig_attestation_signatures_total", C, "Total number of individual attestation signatures", &[], NONE),
    m("lean_pq_sig_attestation_signatures_valid_total", C, "Total number of valid individual attestation signatures", &[], NONE),
    m("lean_pq_sig_attestation_signatures_invalid_total", C, "Total number of invalid individual attestation signatures", &[], NONE),
    m("lean_pq_sig_attestation_signing_time_seconds", H, "Time taken to sign an attestation", &[], &[0.005, 0.01, 0.025, 0.05, 0.1, 1.0]),
    m("lean_pq_sig_attestation_verification_time_seconds", H, "Time taken to verify an attestation signature", &[], FAST),
    m("lean_pq_sig_aggregated_signatures_total", C, "Total number of aggregated signatures", &[], NONE),
    m("lean_pq_sig_aggregated_signatures_valid_total", C, "Total number of valid aggregated signatures", &[], NONE),
    m("lean_pq_sig_aggregated_signatures_invalid_total", C, "Total number of invalid aggregated signatures", &[], NONE),
    m("lean_pq_sig_attestations_in_aggregated_signatures_total", C, "Total number of attestations included into aggregated signatures", &[], NONE),
    m("lean_pq_sig_aggregated_signatures_building_time_seconds", H, "Time taken to build an aggregated attestation signature", &[], AGG),
    m("lean_pq_sig_aggregated_signatures_verification_time_seconds", H, "Time taken to verify an aggregated attestation signature", &[], AGG),
    m("lean_block_aggregated_payloads", H, "Number of aggregated_payloads in a block", &[], &[1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0]),
    m("lean_block_building_payload_aggregation_time_seconds", H, "Time taken to build aggregated_payloads during block building", &[], &[0.1, 0.25, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0]),
    m("lean_block_building_time_seconds", H, "Time taken to build a block", &[], &[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 0.75, 1.0]),
    m("lean_block_building_success_total", C, "Successful block builds", &[], NONE),
    m("lean_block_building_failures_total", C, "Failed block builds (exception in build_block)", &[], NONE),
    m("lean_head_slot", G, "Latest slot of the lean chain", &[], NONE),
    m("lean_current_slot", G, "Current slot of the lean chain", &[], NONE),
    m("lean_safe_target_slot", G, "Safe target slot", &[], NONE),
    m("lean_fork_choice_block_processing_time_seconds", H, "Time taken to process block", &[], &[0.005, 0.01, 0.025, 0.05, 0.1, 1.0, 1.25, 1.5, 2.0, 4.0]),
    m("lean_attestations_valid_total", C, "Total number of valid attestations", &[], NONE),
    m("lean_attestations_invalid_total", C, "Total number of invalid attestations", &[], NONE),
    m("lean_attestation_validation_time_seconds", H, "Time taken to validate attestation", &[], FAST),
    m("lean_fork_choice_reorgs_total", C, "Total number of fork choice reorgs", &[], NONE),
    m("lean_fork_choice_reorg_depth", H, "Depth of fork choice reorgs (in blocks)", &[], &[1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 20.0, 30.0, 50.0, 100.0]),
    m("lean_gossip_signatures", G, "Number of gossip signatures in fork-choice store", &[], NONE),
    m("lean_latest_new_aggregated_payloads", G, "Number of new aggregated payload items", &[], NONE),
    m("lean_latest_known_aggregated_payloads", G, "Number of known aggregated payload items", &[], NONE),
    m("lean_committee_signatures_aggregation_time_seconds", H, "Time taken to aggregate committee signatures", &[], &[0.05, 0.1, 0.25, 0.5, 0.75, 1.0, 2.0, 3.0, 4.0]),
    m("lean_node_sync_status", G, "Node sync status", &["status"], NONE),
    m("lean_tick_interval_duration_seconds", H, "Elapsed time between clock ticks in seconds", &[], &[0.4, 0.6, 0.75, 0.8, 0.805, 0.81, 0.815, 0.82, 0.825, 0.85, 0.9, 1.0, 1.2, 1.6]),
    m("lean_latest_justified_slot", G, "Latest justified slot", &[], NONE),
    m("lean_latest_finalized_slot", G, "Latest finalized slot", &[], NONE),
    m("lean_justified_slot", G, "Current justified slot", &[], NONE),
    m("lean_finalized_slot", G, "Current finalized slot", &[], NONE),
    m("lean_finalizations_total", C, "Total number of finalization attempts", &["result"], NONE),
    m("lean_state_transition_time_seconds", H, "Time to process state transition", &[], &[0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0]),
    m("lean_state_transition_slots_processed_total", C, "Total number of processed slots", &[], NONE),
    m("lean_state_transition_slots_processing_time_seconds", H, "Time taken to process slots", &[], FAST),
    m("lean_state_transition_block_processing_time_seconds", H, "Time taken to process block", &[], FAST),
    m("lean_state_transition_attestations_processed_total", C, "Total number of processed attestations", &[], NONE),
    m("lean_state_transition_attestations_processing_time_seconds", H, "Time taken to process attestations", &[], FAST),
    m("lean_validators_count", G, "Number of validators managed by a node", &[], NONE),
    m("lean_is_aggregator", G, "Validator's is_aggregator status. True=1, False=0", &[], NONE),
    m("lean_attestations_production_time_seconds", H, "Time taken to produce attestation", &[], &[0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 0.75, 1.0]),
    m("lean_aggregator_skipped_total", C, "Total number of aggregation submissions skipped, labeled by reason", &["reason"], NONE),
    m("lean_connected_peers", G, "Number of connected peers", &["client"], NONE),
    m("lean_peer_connection_events_total", C, "Total number of peer connection events", &["direction", "result"], NONE),
    m("lean_peer_disconnection_events_total", C, "Total number of peer disconnection events", &["direction", "reason"], NONE),
    m("lean_gossip_mesh_peers", G, "Number of peers in the gossipsub mesh", &["client"], NONE),
    m("lean_attestation_committee_subnet", G, "Node's attestation committee subnet", &[], NONE),
    m("lean_attestation_committee_count", G, "Number of attestation committees (ATTESTATION_COMMITTEE_COUNT)", &[], NONE),
    m("lean_gossip_block_size_bytes", H, "Bytes size of a gossip block message", &[], &[10000.0, 50000.0, 100000.0, 250000.0, 500000.0, 1000000.0, 2000000.0, 5000000.0]),
    m("lean_gossip_attestation_size_bytes", H, "Bytes size of a gossip attestation message", &[], &[512.0, 1024.0, 2048.0, 4096.0, 8192.0, 16384.0]),
    m("lean_gossip_aggregation_size_bytes", H, "Bytes size of a gossip aggregated attestation message", &[], &[1024.0, 4096.0, 16384.0, 65536.0, 131072.0, 262144.0, 524288.0, 1048576.0]),
    m("lean_gossip_block_arrival_delay_seconds", H, "Absolute delay between a gossip block's arrival and the start of the interval it was due in", &[], ARRIVAL),
    m("lean_gossip_attestation_arrival_delay_seconds", H, "Absolute delay between a gossip attestation's arrival and the start of the interval it was due in", &[], ARRIVAL),
    m("lean_gossip_aggregation_arrival_delay_seconds", H, "Absolute delay between a gossip aggregate's arrival and the most recent aggregation-interval boundary at or before it", &[], ARRIVAL),
    m("lean_gossip_block_arrival_total", C, "Gossip blocks by arrival position relative to the interval they were due in", &["position"], NONE),
    m("lean_gossip_attestation_arrival_total", C, "Gossip attestations by arrival position relative to the interval they were due in", &["position"], NONE),
    m("lean_gossip_aggregation_arrival_total", C, "Gossip aggregates by arrival position relative to the most recent aggregation-interval boundary", &["position"], NONE),
];

/// Look up a metric by name.
pub fn spec(name: &str) -> Option<&'static LeanSpec> {
    LEAN_METRICS
        .iter()
        .chain(super::spec_extra::LEAN_METRICS_EXTRA.iter())
        .find(|s| s.name == name)
}
