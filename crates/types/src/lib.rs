//! Lean Consensus domain types (lstar containers).

#![forbid(unsafe_code)]

mod aggregate;
mod checkpoint;
mod error;
mod genesis;
mod limits;
mod validator;

pub mod block;
pub mod operation;
pub mod proofs;
pub mod state;

pub use aggregate::{AggregationBits, MultiMessageAggregate, SingleMessageAggregate};
pub use block::{Block, BlockBody, BlockHeader, SignedBlock};
pub use checkpoint::{AttestationData, Checkpoint};
pub use error::TypesError;
pub use genesis::GenesisConfig;
pub use limits::{
    AGGREGATED_ATTESTATIONS_LIMIT, BYTE_LIST_512_KIB, HISTORICAL_ROOTS_LIMIT,
    JUSTIFICATION_VALIDATORS_LIMIT, MAX_ATTESTATIONS_DATA, VALIDATOR_REGISTRY_LIMIT,
    XMSS_PUBLIC_KEY_BYTES, XMSS_SIGNATURE_BYTES,
};
pub use operation::{
    AggregatedAttestation, Attestation, SignedAggregatedAttestation, SignedAttestation,
};
pub use proofs::{indices_from_bits, validate_ordered_indices};
pub use state::State;
pub use validator::Validator;

/// Alias used by storage and API layers for 32-byte roots.
pub type Root = Hash32;
/// Alias for state roots.
pub type StateRoot = Hash32;

pub use ethean_primitives::{Bytes52, Epoch, Hash32, Slot, ValidatorIndex, HASH32_ZERO};
