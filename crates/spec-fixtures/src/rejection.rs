//! Map leanSpec `rejectionReason` strings to fork-choice errors.

use ethean_fork_choice::ForkChoiceError;

/// Known leanSpec rejection token for fork-choice vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkChoiceRejection {
    /// `BLOCK_TOO_FAR_IN_FUTURE`
    BlockTooFarInFuture,
    /// `BLOCK_SLOT_GAP_TOO_LARGE`
    BlockSlotGapTooLarge,
    /// `UNKNOWN_PARENT` / `UNKNOWN_PARENT_BLOCK`
    UnknownParent,
    /// `UNKNOWN_SOURCE_BLOCK`
    UnknownSourceBlock,
    /// `UNKNOWN_TARGET_BLOCK`
    UnknownTargetBlock,
    /// `UNKNOWN_HEAD_BLOCK`
    UnknownHeadBlock,
    /// `SOURCE_AFTER_TARGET` / similar
    SourceAfterTarget,
    /// `HEAD_OLDER_THAN_TARGET`
    HeadOlderThanTarget,
    /// `CHECKPOINT_SLOT_MISMATCH`
    CheckpointSlotMismatch,
    /// `SOURCE_NOT_ANCESTOR_OF_TARGET`
    SourceNotAncestorOfTarget,
    /// `TARGET_NOT_ANCESTOR_OF_HEAD`
    TargetNotAncestorOfHead,
    /// `HEAD_NOT_DESCENDANT_OF_FINALIZED`
    HeadNotDescendantOfFinalized,
    /// `ATTESTATION_SLOT_BEFORE_HEAD`
    AttestationSlotBeforeHead,
    /// `ATTESTATION_TOO_FAR_IN_FUTURE`
    AttestationTooFarInFuture,
    /// `DUPLICATE_ATTESTATION_DATA`
    DuplicateAttestationData,
}

/// Sentinel when the fixture string is not yet mapped.
pub const UNKNOWN_REJECTION: &str = "UNMAPPED_REJECTION_REASON";

/// Parse a leanSpec rejection reason into a typed fork-choice token.
pub fn map_fork_choice_rejection(reason: &str) -> Option<ForkChoiceRejection> {
    match reason.trim() {
        "BLOCK_TOO_FAR_IN_FUTURE" => Some(ForkChoiceRejection::BlockTooFarInFuture),
        "BLOCK_SLOT_GAP_TOO_LARGE" => Some(ForkChoiceRejection::BlockSlotGapTooLarge),
        "UNKNOWN_PARENT" | "UNKNOWN_PARENT_BLOCK" => Some(ForkChoiceRejection::UnknownParent),
        "UNKNOWN_SOURCE_BLOCK" => Some(ForkChoiceRejection::UnknownSourceBlock),
        "UNKNOWN_TARGET_BLOCK" => Some(ForkChoiceRejection::UnknownTargetBlock),
        "UNKNOWN_HEAD_BLOCK" => Some(ForkChoiceRejection::UnknownHeadBlock),
        "SOURCE_AFTER_TARGET" | "SOURCE_SLOT_EXCEEDS_TARGET" => {
            Some(ForkChoiceRejection::SourceAfterTarget)
        }
        "HEAD_OLDER_THAN_TARGET" => Some(ForkChoiceRejection::HeadOlderThanTarget),
        "CHECKPOINT_SLOT_MISMATCH"
        | "HEAD_SLOT_MISMATCH"
        | "TARGET_SLOT_MISMATCH"
        | "SOURCE_SLOT_MISMATCH" => Some(ForkChoiceRejection::CheckpointSlotMismatch),
        "SOURCE_NOT_ANCESTOR_OF_TARGET" => Some(ForkChoiceRejection::SourceNotAncestorOfTarget),
        "TARGET_NOT_ANCESTOR_OF_HEAD" | "HEAD_ON_SIBLING_FORK" => {
            Some(ForkChoiceRejection::TargetNotAncestorOfHead)
        }
        "HEAD_NOT_DESCENDANT_OF_FINALIZED" => {
            Some(ForkChoiceRejection::HeadNotDescendantOfFinalized)
        }
        "ATTESTATION_SLOT_BEFORE_HEAD" => Some(ForkChoiceRejection::AttestationSlotBeforeHead),
        "ATTESTATION_TOO_FAR_IN_FUTURE" => Some(ForkChoiceRejection::AttestationTooFarInFuture),
        "DUPLICATE_ATTESTATION_DATA" => Some(ForkChoiceRejection::DuplicateAttestationData),
        _ => None,
    }
}

impl ForkChoiceRejection {
    /// Convert to the live [`ForkChoiceError`] variant.
    pub fn to_error(self) -> ForkChoiceError {
        match self {
            Self::BlockTooFarInFuture => ForkChoiceError::BlockTooFarInFuture,
            Self::BlockSlotGapTooLarge => ForkChoiceError::BlockSlotGapTooLarge,
            Self::UnknownParent => ForkChoiceError::UnknownParent,
            Self::UnknownSourceBlock => ForkChoiceError::UnknownSourceBlock,
            Self::UnknownTargetBlock => ForkChoiceError::UnknownTargetBlock,
            Self::UnknownHeadBlock => ForkChoiceError::UnknownHeadBlock,
            Self::SourceAfterTarget => ForkChoiceError::SourceAfterTarget,
            Self::HeadOlderThanTarget => ForkChoiceError::HeadOlderThanTarget,
            Self::CheckpointSlotMismatch => ForkChoiceError::CheckpointSlotMismatch,
            Self::SourceNotAncestorOfTarget => ForkChoiceError::SourceNotAncestorOfTarget,
            Self::TargetNotAncestorOfHead => ForkChoiceError::TargetNotAncestorOfHead,
            Self::HeadNotDescendantOfFinalized => ForkChoiceError::HeadNotDescendantOfFinalized,
            Self::AttestationSlotBeforeHead => ForkChoiceError::AttestationSlotBeforeHead,
            Self::AttestationTooFarInFuture => ForkChoiceError::AttestationTooFarInFuture,
            Self::DuplicateAttestationData => ForkChoiceError::DuplicateAttestationData,
        }
    }

    /// Primary leanSpec wire token for this variant.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BlockTooFarInFuture => "BLOCK_TOO_FAR_IN_FUTURE",
            Self::BlockSlotGapTooLarge => "BLOCK_SLOT_GAP_TOO_LARGE",
            Self::UnknownParent => "UNKNOWN_PARENT_BLOCK",
            Self::UnknownSourceBlock => "UNKNOWN_SOURCE_BLOCK",
            Self::UnknownTargetBlock => "UNKNOWN_TARGET_BLOCK",
            Self::UnknownHeadBlock => "UNKNOWN_HEAD_BLOCK",
            Self::SourceAfterTarget => "SOURCE_AFTER_TARGET",
            Self::HeadOlderThanTarget => "HEAD_OLDER_THAN_TARGET",
            Self::CheckpointSlotMismatch => "CHECKPOINT_SLOT_MISMATCH",
            Self::SourceNotAncestorOfTarget => "SOURCE_NOT_ANCESTOR_OF_TARGET",
            Self::TargetNotAncestorOfHead => "TARGET_NOT_ANCESTOR_OF_HEAD",
            Self::HeadNotDescendantOfFinalized => "HEAD_NOT_DESCENDANT_OF_FINALIZED",
            Self::AttestationSlotBeforeHead => "ATTESTATION_SLOT_BEFORE_HEAD",
            Self::AttestationTooFarInFuture => "ATTESTATION_TOO_FAR_IN_FUTURE",
            Self::DuplicateAttestationData => "DUPLICATE_ATTESTATION_DATA",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_block_too_far() {
        let r = map_fork_choice_rejection("BLOCK_TOO_FAR_IN_FUTURE").unwrap();
        assert_eq!(r.to_error(), ForkChoiceError::BlockTooFarInFuture);
    }

    #[test]
    fn maps_attestation_reasons() {
        assert_eq!(
            map_fork_choice_rejection("UNKNOWN_SOURCE_BLOCK")
                .unwrap()
                .to_error(),
            ForkChoiceError::UnknownSourceBlock
        );
        assert_eq!(
            map_fork_choice_rejection("ATTESTATION_TOO_FAR_IN_FUTURE")
                .unwrap()
                .to_error(),
            ForkChoiceError::AttestationTooFarInFuture
        );
    }

    #[test]
    fn unknown_stays_none() {
        assert!(map_fork_choice_rejection("NOT_A_REAL_REASON").is_none());
    }
}
