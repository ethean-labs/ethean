//! Map leanSpec `rejectionReason` strings to fork-choice errors.

use ethean_fork_choice::ForkChoiceError;

/// Known leanSpec rejection token for fork-choice vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkChoiceRejection {
    /// `BLOCK_TOO_FAR_IN_FUTURE`
    BlockTooFarInFuture,
    /// `BLOCK_SLOT_GAP_TOO_LARGE` (historical roots limit)
    BlockSlotGapTooLarge,
    /// `UNKNOWN_PARENT`
    UnknownParent,
}

/// Sentinel when the fixture string is not yet mapped.
pub const UNKNOWN_REJECTION: &str = "UNMAPPED_REJECTION_REASON";

/// Parse a leanSpec rejection reason into a typed fork-choice token.
pub fn map_fork_choice_rejection(reason: &str) -> Option<ForkChoiceRejection> {
    match reason.trim() {
        "BLOCK_TOO_FAR_IN_FUTURE" => Some(ForkChoiceRejection::BlockTooFarInFuture),
        "BLOCK_SLOT_GAP_TOO_LARGE" => Some(ForkChoiceRejection::BlockSlotGapTooLarge),
        "UNKNOWN_PARENT" => Some(ForkChoiceRejection::UnknownParent),
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
        }
    }

    /// leanSpec wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BlockTooFarInFuture => "BLOCK_TOO_FAR_IN_FUTURE",
            Self::BlockSlotGapTooLarge => "BLOCK_SLOT_GAP_TOO_LARGE",
            Self::UnknownParent => "UNKNOWN_PARENT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_block_too_far() {
        let r = map_fork_choice_rejection("BLOCK_TOO_FAR_IN_FUTURE").unwrap();
        assert_eq!(r, ForkChoiceRejection::BlockTooFarInFuture);
        assert_eq!(r.to_error(), ForkChoiceError::BlockTooFarInFuture);
    }

    #[test]
    fn unknown_stays_none() {
        assert!(map_fork_choice_rejection("NOT_A_REAL_REASON").is_none());
    }
}
