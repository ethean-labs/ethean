//! Attestation operations and justification updates.

pub mod attestation;
pub mod justify;

pub use attestation::{
    check_attestation_data_structure, distinct_attestation_data_count, process_attestations,
};
