//! Lean Consensus sync: status gate, parent/range, backfill, checkpoint trust.

#![forbid(unsafe_code)]

pub mod backfill;
pub mod checkpoint;
pub mod error;
pub mod parent;
pub mod range;
pub mod status;
pub mod trust;

pub use backfill::{validate_backfill, BackfillJob};
pub use checkpoint::CheckpointBundle;
pub use error::{Result, SyncError};
pub use parent::{plan_parent_sync, ParentRequest};
pub use range::{plan_range, RangeBatch};
pub use status::{SyncMode, SyncStatus};
pub use trust::{evaluate_trust, TrustPolicy};
