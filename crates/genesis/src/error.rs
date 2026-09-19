//! Genesis and clock errors.

use thiserror::Error;

/// Errors from wall-clock acquisition and slot mapping.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ClockError {
    /// Current time is before genesis.
    #[error("time is before genesis")]
    PreGenesis,

    /// Checked multiply/add overflow while converting time or slots.
    #[error("timestamp arithmetic overflow")]
    TimestampOverflow,

    /// Injected time moved backwards while regression is rejected.
    #[error("time moved backwards")]
    BackwardTime,

    /// Underlying system clock is unavailable or before Unix epoch.
    #[error("system clock unavailable")]
    SystemClockUnavailable,

    /// Slot duration or interval duration is zero (invalid profile).
    #[error("zero duration in profile timing")]
    ZeroDuration,
}

/// Errors from genesis construction, loading, and profile checks.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GenesisError {
    #[error(transparent)]
    Clock(#[from] ClockError),

    #[error("chain profile mismatch: {0}")]
    ProfileMismatch(String),

    #[error("network genesis requires at least one validator")]
    EmptyValidators,

    #[error("invalid validator index {index} (expected {expected})")]
    InvalidValidatorIndex { index: u64, expected: u64 },

    #[error("genesis state root mismatch")]
    GenesisRootMismatch,

    #[error("genesis SSZ payload is empty or truncated")]
    TruncatedOrEmpty,

    #[error("types error: {0}")]
    Types(String),

    #[error("profile error: {0}")]
    Profile(String),
}
