//! Metrics errors.

use thiserror::Error;

/// Observability failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MetricsError {
    #[error("forbidden label: {0}")]
    ForbiddenLabel(String),

    #[error("duplicate metric registration: {0}")]
    DuplicateMetric(String),

    #[error("unknown metric: {0}")]
    UnknownMetric(String),

    #[error("not ready: {0}")]
    NotReady(String),
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, MetricsError>;
