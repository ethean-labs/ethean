//! Ethean observability: registry, export, readiness (leanMetrics pin OSD-009 open).

#![forbid(unsafe_code)]

pub mod error;
pub mod export;
pub mod families;
pub mod http;
pub mod readiness;
pub mod record;
pub mod registry;
pub mod schema;
pub mod shared;

pub use error::{MetricsError, Result};
pub use export::export_prometheus_text;
pub use http::spawn_metrics_server;
pub use readiness::Readiness;
pub use record::{
    ensure_core_families, record_readiness_gauges, record_role_gauges, record_slot_gauges, set_ready,
};
pub use registry::{MetricKind, MetricSample, Registry};
pub use schema::{assert_label_allowed, FORBIDDEN_LABELS, METRIC_PREFIX, METRICS_SCHEMA_VERSION};
pub use shared::SharedRegistry;
