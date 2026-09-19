//! Metrics errors.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MetricsError {
    #[error("metrics not implemented (Phase 12); leanMetrics pin open (OSD-009)")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, MetricsError>;
