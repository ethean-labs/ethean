//! leanSpec fixture envelopes and Hive-oriented discovery for Ethean.
//!
//! Full fork-choice / STF execution against filled vectors is incremental.
//! This crate indexes JSON fixtures, maps leanSpec `rejectionReason` strings,
//! and runs mapped fork-choice rejection steps when anchors decode.

#![forbid(unsafe_code)]

mod discover;
mod envelope;
mod fc_checks;
mod fc_runner;
mod fc_steps;
mod hex;
mod json_types;
mod rejection;

pub use discover::{discover_json_fixtures, fixtures_root_from_env, FIXTURES_ENV};
pub use envelope::{FixtureCase, FixtureFile, FixtureStep};
pub use fc_runner::{
    run_fork_choice_case, run_fork_choice_file, run_fork_choice_rejections, FcRunError,
    FcRunReport,
};
pub use rejection::{
    map_fork_choice_rejection, ForkChoiceRejection, UNKNOWN_REJECTION,
};
