//! Shared helpers for slot math, justified bits, and proposer schedule.

pub mod justified_bits;
pub mod proposer;
pub mod slot_math;

pub use justified_bits::{extend_to_slot, is_slot_justified};
pub use proposer::{index_within_registry, proposer_for_slot};
pub use slot_math::{is_justifiable_after, justified_index_after};
