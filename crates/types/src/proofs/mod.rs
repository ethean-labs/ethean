//! Consensus-side proof helpers (participant index handling).

mod participants;

pub use participants::{indices_from_bits, validate_ordered_indices};
