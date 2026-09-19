//! Type-1 / Type-2 aggregate proof verification (Phase 08).
//!
//! Production leanVM verify fails closed until the pinned backend compiles.
//! `test-aggregate` provides a statement-bound synthetic verifier for unit tests
//! (never always-true on arbitrary bytes).

mod bindings;
mod prove;
mod statement;
mod statement_wire;
mod verify;

pub use bindings::{
    assert_aggregation_invariants, aggregation_fingerprint, LEANVM_REV, LOG_INV_RATE,
    MAX_PROOF_BYTES, MAX_TYPE2_COMPONENTS, PROD_AGGREGATION_FINGERPRINT,
};
pub use prove::{prove_type1, prove_type2};
pub use statement::{
    AggregateStatement, ParticipantSet, ProofKind, Type2ComponentRef, MAX_PARTICIPANTS,
};
pub use verify::{verify_statement_shape, verify_type1, verify_type2};
