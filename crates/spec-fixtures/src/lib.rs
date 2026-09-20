//! leanSpec fixture envelopes and Hive-oriented discovery for Ethean.
//!
//! Full fork-choice / STF execution against filled vectors is incremental.
//! This crate indexes JSON fixtures and maps leanSpec `rejectionReason`
//! strings onto [`ForkChoiceError`] so matrix work can start before every
//! step runner lands.

#![forbid(unsafe_code)]

mod discover;
mod envelope;
mod rejection;

pub use discover::{discover_json_fixtures, fixtures_root_from_env, FIXTURES_ENV};
pub use envelope::{FixtureCase, FixtureFile, FixtureStep};
pub use rejection::{
    map_fork_choice_rejection, ForkChoiceRejection, UNKNOWN_REJECTION,
};
