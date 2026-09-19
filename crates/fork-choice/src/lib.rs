//! Lean Consensus fork-choice store (lstar / modified 3SF-mini).
//!
//! Authority: leanSpec@0b7d33ec `fork_choice.py`, `timeline.py`, `containers/store.py`.
//! Phase 00 resolved generation: **not Goldfish**.

#![forbid(unsafe_code)]

mod error;
mod head;
mod opts;
mod prune;
mod store;
mod update;

pub use error::ForkChoiceError;
pub use opts::ForkChoiceOpts;
pub use store::ForkChoiceStore;
pub use update::create_store;

pub use ethean_primitives::{Hash32, ValidatorIndex};
pub use ethean_types::{AttestationData, Block, Checkpoint, State};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
