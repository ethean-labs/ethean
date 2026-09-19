//! Consensus-side proof helpers (participants / Type-1 / Type-2 statements).

mod participants;
mod type1;
mod type2;

pub use participants::{indices_from_bits, validate_ordered_indices};
pub use type1::type1_statement_from_aggregate;
pub use type2::{type2_statement_for_block, Type2ComponentRoots};
