//! Transition rejection reasons aligned with leanSpec `SpecRejectionError`.

use thiserror::Error;

/// Errors raised while applying Lean state transition.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransitionError {
    #[error("BLOCK_SLOT_NOT_IN_FUTURE: {0}")]
    BlockSlotNotInFuture(String),

    #[error("BLOCK_SLOT_MISMATCH: {0}")]
    BlockSlotMismatch(String),

    #[error("BLOCK_OLDER_THAN_LATEST_HEADER: {0}")]
    BlockOlderThanLatestHeader(String),

    #[error("PARENT_ROOT_MISMATCH: {0}")]
    InvalidParent(String),

    #[error("WRONG_PROPOSER: {0}")]
    WrongProposer(String),

    #[error("PROPOSER_INDEX_OUT_OF_RANGE: {0}")]
    ProposerIndexOutOfRange(String),

    #[error("EMPTY_VALIDATOR_REGISTRY: {0}")]
    EmptyValidatorRegistry(String),

    #[error("TOO_MANY_ATTESTATION_DATA: {0}")]
    AttestationDataLimit(String),

    #[error("STATE_ROOT_MISMATCH: {0}")]
    InvalidStateRoot(String),

    #[error("INVALID_BLOCK_PROOF: {0}")]
    InvalidBlockProof(String),
    #[error("UNSUPPORTED_SIGNATURE: {0}")]
    UnsupportedSignature(String),

    #[error("EMPTY_AGGREGATION_BITS: {0}")]
    EmptyAggregationBits(String),

    #[error("VALIDATOR_INDEX_OUT_OF_RANGE: {0}")]
    ValidatorIndexOutOfRange(String),

    #[error("JUSTIFICATION_VOTES_LENGTH_MISMATCH: {0}")]
    JustificationVotesLengthMismatch(String),

    #[error("ZERO_HASH_JUSTIFICATION_ROOT: {0}")]
    ZeroHashJustificationRoot(String),

    #[error("JUSTIFIED_SLOT_OUT_OF_RANGE: {0}")]
    JustifiedSlotOutOfRange(String),

    #[error("types: {0}")]
    Types(String),
}
