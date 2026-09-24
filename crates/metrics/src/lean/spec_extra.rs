//! Gauges the leanSpec node registry (`node/metrics/registry.py`) adds on top
//! of the pinned leanMetrics table; hive's API scrape contract requires them.

use super::spec::{LeanKind, LeanSpec};

/// Attestation aggregate coverage sections (`ATTESTATION_AGGREGATE_COVERAGE_SECTIONS`).
pub const COVERAGE_SECTIONS: &[&str] = &["timely", "late", "block", "combined"];
/// Coverage delta directions between block payloads and timely pre-merge payloads.
pub const COVERAGE_DIFF_DIRECTIONS: &[&str] = &["block_only", "timely_only"];

/// Extra series from the leanSpec node registry.
pub const LEAN_METRICS_EXTRA: &[LeanSpec] = &[
    LeanSpec {
        name: "lean_attestation_aggregate_coverage_validators",
        kind: LeanKind::Gauge,
        help: "Validator coverage in attestation aggregate reports, by section and subnet",
        labels: &["section", "subnet"],
        buckets: &[],
    },
    LeanSpec {
        name: "lean_attestation_aggregate_coverage_subnets",
        kind: LeanKind::Gauge,
        help: "Number of covered subnets in attestation aggregate reports, by section",
        labels: &["section"],
        buckets: &[],
    },
    LeanSpec {
        name: "lean_attestation_aggregate_coverage_diff_validators",
        kind: LeanKind::Gauge,
        help: "Validator coverage delta between block payloads and timely pre-merge payloads",
        labels: &["direction"],
        buckets: &[],
    },
];
