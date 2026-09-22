//! Aggregation pools: verified individual signatures awaiting aggregation and
//! verified Type-1 proofs ready for block building.

mod pool;
mod signatures;

pub use pool::{AggregatePool, PoolEntry, PoolKey};
pub use signatures::{AttestationSignaturePool, MAX_TRACKED_DATA};
