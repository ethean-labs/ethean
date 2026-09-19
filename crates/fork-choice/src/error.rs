//! Fork-choice error types.

use thiserror::Error;

/// Errors from store updates and head selection.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ForkChoiceError {
    #[error("anchor block state root does not match state hash tree root")]
    AnchorStateRootMismatch,

    #[error("unknown parent block")]
    UnknownParent,

    #[error("unknown source block")]
    UnknownSourceBlock,

    #[error("unknown target block")]
    UnknownTargetBlock,

    #[error("unknown head block")]
    UnknownHeadBlock,

    #[error("source checkpoint slot after target")]
    SourceAfterTarget,

    #[error("head checkpoint older than target")]
    HeadOlderThanTarget,

    #[error("checkpoint slot does not match block slot")]
    CheckpointSlotMismatch,

    #[error("source is not an ancestor of target")]
    SourceNotAncestorOfTarget,

    #[error("target is not an ancestor of head")]
    TargetNotAncestorOfHead,

    #[error("head is not a descendant of the finalized checkpoint")]
    HeadNotDescendantOfFinalized,

    #[error("attestation slot precedes head")]
    AttestationSlotBeforeHead,

    #[error("attestation too far in the future")]
    AttestationTooFarInFuture,

    #[error("block slot gap too large")]
    BlockSlotGapTooLarge,

    #[error("block too far in the future")]
    BlockTooFarInFuture,

    #[error("duplicate AttestationData in block body")]
    DuplicateAttestationData,

    #[error("missing block or state for root")]
    MissingBlockOrState,

    #[error("tick target must not be before store time")]
    TickInPast,

    #[error("types / SSZ: {0}")]
    Types(String),

    #[error("signature / aggregate proofs unsupported until Phase 07/08: {0}")]
    UnsupportedSignature(String),
}
