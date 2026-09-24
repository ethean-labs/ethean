//! leanSpec fixture envelopes and Hive-oriented discovery for Ethean.
//!
//! Full fork-choice / STF execution against filled vectors is incremental.
//! This crate indexes JSON fixtures, maps leanSpec `rejectionReason` strings,
//! and runs mapped fork-choice rejection steps when anchors decode.

#![forbid(unsafe_code)]

mod discover;
mod driver;
mod envelope;
mod fc_checks;
mod fc_snapshot_payloads;
mod fc_store_snapshot;
mod fc_runner;
mod fc_steps;
mod hex;
mod json_signed;
mod json_types;
mod rejection;
mod stf_runner;
mod suite_networking;
mod suite_networking_wire;
mod suite_scalar;
mod suite_ssz;

#[cfg(test)]
mod suites_tests;

pub use discover::{discover_json_fixtures, fixtures_root_from_env, FIXTURES_ENV};
pub use driver::{
    apply_driver_step, decode_hex_bytes, driver_snapshot, init_driver_store,
    run_state_transition_driver, DriverStore, VoteEvidence, VoteVerifier,
};
pub use json_signed::{attestation_with_signature_from_value, signed_aggregated_with_proof_from_value};
pub use json_types::{block_from_value, state_from_value};
pub use envelope::{FixtureCase, FixtureFile, FixtureStep};
pub use fc_runner::{
    run_fork_choice_case, run_fork_choice_file, run_fork_choice_rejections, FcRunError,
    FcRunReport,
};
pub use rejection::{
    map_fork_choice_rejection, ForkChoiceRejection, UNKNOWN_REJECTION,
};
pub use stf_runner::{run_state_transition_case, run_state_transition_file, StfRunError, StfRunReport};
pub use suite_networking::{run_networking_case, NetOutcome};
pub use suite_scalar::{
    intervals_since_genesis, run_justifiability_case, run_poseidon_case, run_slot_clock_case,
    run_sync_case,
};
pub use suite_ssz::{run_ssz_case, SszOutcome};
