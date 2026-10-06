//! Votes the fork-choice store could not place yet: the head, target or source
//! block has not been imported, or the vote arrived before the local tick
//! reached its slot. They are retried after block imports and ticks.

use crate::chain_owner::ChainOwner;
use ethean_fork_choice::ForkChoiceError;
use ethean_primitives::ValidatorIndex;
use ethean_types::AttestationData;
use std::collections::VecDeque;
use tracing::debug;

/// Deferred votes kept at most; the oldest is dropped first.
pub const MAX_DEFERRED_VOTES: usize = 1024;

/// Slots a deferred vote may wait behind the store clock before it is dropped.
pub const DEFERRED_VOTE_SLOTS: u64 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeferredVote {
    Single(ValidatorIndex, AttestationData),
    Aggregated(AttestationData, Vec<bool>),
}

impl DeferredVote {
    fn data(&self) -> &AttestationData {
        match self {
            Self::Single(_, data) | Self::Aggregated(data, _) => data,
        }
    }
}

#[derive(Debug, Default)]
pub struct DeferredVotes {
    queue: VecDeque<DeferredVote>,
}

impl DeferredVotes {
    pub fn push(&mut self, vote: DeferredVote) {
        if self.queue.len() >= MAX_DEFERRED_VOTES {
            self.queue.pop_front();
        }
        self.queue.push_back(vote);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Errors that can clear once a block arrives or the store clock moves.
pub fn is_retryable(err: &ForkChoiceError) -> bool {
    matches!(
        err,
        ForkChoiceError::UnknownHeadBlock
            | ForkChoiceError::UnknownTargetBlock
            | ForkChoiceError::UnknownSourceBlock
            | ForkChoiceError::AttestationTooFarInFuture
    )
}

impl ChainOwner {
    /// Replay deferred votes; keeps the ones still waiting on a block or tick.
    pub fn retry_deferred_votes(&mut self) {
        if self.deferred_votes.is_empty() {
            return;
        }
        let Some(fc) = self.fc.as_mut() else {
            return;
        };
        let store_slot = fc.time / fc.intervals_per_slot.max(1);
        let finalized = fc.latest_finalized.slot.get();
        let pending = std::mem::take(&mut self.deferred_votes.queue);
        let mut applied = 0usize;
        let mut dropped = 0usize;
        for vote in pending {
            let slot = vote.data().slot.get();
            if slot <= finalized || slot.saturating_add(DEFERRED_VOTE_SLOTS) < store_slot {
                dropped += 1;
                continue;
            }
            let result = match &vote {
                DeferredVote::Single(validator, data) => fc.on_attestation_data(*validator, *data),
                DeferredVote::Aggregated(data, participants) => {
                    fc.on_aggregated_attestation(*data, participants)
                }
            };
            match result {
                Ok(()) => applied += 1,
                Err(e) if is_retryable(&e) => self.deferred_votes.queue.push_back(vote),
                Err(_) => dropped += 1,
            }
        }
        if applied > 0 || dropped > 0 {
            debug!(
                applied,
                dropped,
                waiting = self.deferred_votes.len(),
                "deferred fork-choice votes retried"
            );
        }
        if applied > 0 {
            self.sync_from_fork_choice();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_types::Checkpoint;

    fn vote(slot: u64) -> DeferredVote {
        let data = AttestationData {
            slot: Slot::new(slot),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        DeferredVote::Single(ValidatorIndex::new(0), data)
    }

    #[test]
    fn queue_drops_the_oldest_when_full() {
        let mut votes = DeferredVotes::default();
        for slot in 0..(MAX_DEFERRED_VOTES as u64 + 5) {
            votes.push(vote(slot));
        }
        assert_eq!(votes.len(), MAX_DEFERRED_VOTES);
        assert_eq!(votes.queue.front(), Some(&vote(5)));
    }

    fn owner_with_store() -> ChainOwner {
        use ethean_primitives::Bytes52;
        use ethean_types::{BlockBody, BlockHeader, GenesisConfig, State, Validator};
        let val = Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap();
        let mut state = State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vec![val],
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        };
        state.latest_block_header.body_root = BlockBody::default().hash_tree_root().unwrap();
        let mut owner = ChainOwner::new(2);
        owner.head_state = Some(state);
        owner.profile = Some(crate::lstar_devnet().unwrap());
        owner.try_init_fork_choice();
        owner
    }

    #[test]
    fn early_vote_is_applied_once_the_store_reaches_its_slot() {
        let mut owner = owner_with_store();
        let anchor = Checkpoint {
            root: owner.fc.as_ref().unwrap().head(),
            slot: Slot::ZERO,
        };
        let data = AttestationData {
            slot: Slot::new(5),
            head: anchor,
            target: anchor,
            source: anchor,
        };
        owner.fc_on_attestation(ValidatorIndex::new(0), data);
        assert_eq!(owner.deferred_votes.len(), 1);

        owner.fc_on_tick(5, 0, false);
        assert!(owner.deferred_votes.is_empty());
        let fc = owner.fc.as_ref().unwrap();
        let known = fc
            .latest_new_attestations
            .get(&ValidatorIndex::new(0))
            .or_else(|| fc.latest_known_attestations.get(&ValidatorIndex::new(0)));
        assert_eq!(known, Some(&data));
    }

    #[test]
    fn stale_deferred_votes_are_dropped() {
        let mut owner = owner_with_store();
        owner.deferred_votes.push(vote(1));
        owner.fc_on_tick(1 + DEFERRED_VOTE_SLOTS + 1, 0, false);
        assert!(owner.deferred_votes.is_empty());
    }

    #[test]
    fn only_block_and_clock_errors_are_retried() {
        assert!(is_retryable(&ForkChoiceError::UnknownHeadBlock));
        assert!(is_retryable(&ForkChoiceError::AttestationTooFarInFuture));
        assert!(!is_retryable(&ForkChoiceError::SourceAfterTarget));
        assert!(!is_retryable(&ForkChoiceError::ValidatorNotInState));
    }
}
