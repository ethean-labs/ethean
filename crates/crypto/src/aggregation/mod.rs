//! Aggregate-proof parameters and the verifier interface.
//!
//! Proof generation and verification live in `ethean-multisig` (leanMultisig);
//! this crate only defines what consensus code needs to call it.

mod bindings;
mod verifier;

pub use bindings::{
    aggregation_fingerprint, assert_aggregation_invariants, LEANVM_REV, LOG_INV_RATE,
    MAX_PROOF_BYTES, MAX_TYPE2_COMPONENTS, PROD_AGGREGATION_FINGERPRINT,
};
pub use verifier::{AggregateVerifier, ProofComponent};
