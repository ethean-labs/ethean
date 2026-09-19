//! Observability and leanMetrics hooks (Phase 12; OSD-009 still open).

#![forbid(unsafe_code)]

pub mod error;

pub use error::{MetricsError, Result};

/// Placeholder metrics facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct MetricsFacade;
