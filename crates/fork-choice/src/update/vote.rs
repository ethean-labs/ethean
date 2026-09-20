//! Attestation / vote ingest.

use ethean_primitives::ValidatorIndex;
use ethean_types::AttestationData;

use crate::error::ForkChoiceError;
use crate::store::ForkChoiceStore;

impl ForkChoiceStore {
    /// Structural vote ingest into the pending pool.
    ///
    /// When `opts.require_proofs` is true, returns
    /// [`ForkChoiceError::UnsupportedSignature`] (no fake accept on production path).
    pub fn on_attestation_data(
        &mut self,
        validator: ValidatorIndex,
        data: AttestationData,
    ) -> Result<(), ForkChoiceError> {
        if self.opts.require_proofs {
            return Err(ForkChoiceError::UnsupportedSignature(
                "attestation proofs / XMSS verify deferred to Phase 07/08".into(),
            ));
        }
        self.validate_attestation(&data)?;
        self.insert_pending_vote(validator, data);
        Ok(())
    }

    /// Structural gossip aggregate: validate once, then pending-vote each participant.
    ///
    /// Proof bytes are ignored on the structural path; empty participation is rejected.
    pub fn on_aggregated_attestation(
        &mut self,
        data: AttestationData,
        participants: &[bool],
    ) -> Result<(), ForkChoiceError> {
        if self.opts.require_proofs {
            return Err(ForkChoiceError::UnsupportedSignature(
                "aggregate proofs / XMSS verify deferred to Phase 07/08".into(),
            ));
        }
        if !participants.iter().any(|b| *b) {
            return Err(ForkChoiceError::EmptyAggregationBits);
        }
        self.validate_attestation(&data)?;
        for (i, bit) in participants.iter().enumerate() {
            if *bit {
                self.insert_pending_vote(ValidatorIndex::new(i as u64), data.clone());
            }
        }
        Ok(())
    }

    fn insert_pending_vote(&mut self, validator: ValidatorIndex, data: AttestationData) {
        match self.latest_new_attestations.get(&validator) {
            Some(existing) if !vote_is_newer(existing, &data) => {}
            _ => {
                self.latest_new_attestations.insert(validator, data);
            }
        }
    }

    pub(crate) fn validate_attestation(
        &self,
        data: &AttestationData,
    ) -> Result<(), ForkChoiceError> {
        if !self.blocks.contains_key(&data.source.root) {
            return Err(ForkChoiceError::UnknownSourceBlock);
        }
        if !self.blocks.contains_key(&data.target.root) {
            return Err(ForkChoiceError::UnknownTargetBlock);
        }
        if !self.blocks.contains_key(&data.head.root) {
            return Err(ForkChoiceError::UnknownHeadBlock);
        }
        if data.source.slot > data.target.slot {
            return Err(ForkChoiceError::SourceAfterTarget);
        }
        if data.head.slot < data.target.slot {
            return Err(ForkChoiceError::HeadOlderThanTarget);
        }
        if self.blocks[&data.source.root].slot != data.source.slot
            || self.blocks[&data.target.root].slot != data.target.slot
            || self.blocks[&data.head.root].slot != data.head.slot
        {
            return Err(ForkChoiceError::CheckpointSlotMismatch);
        }
        if !self.checkpoint_is_ancestor(data.source, data.target) {
            return Err(ForkChoiceError::SourceNotAncestorOfTarget);
        }
        if !self.checkpoint_is_ancestor(data.target, data.head) {
            return Err(ForkChoiceError::TargetNotAncestorOfHead);
        }
        if !self.checkpoint_is_ancestor(self.latest_finalized, data.head) {
            return Err(ForkChoiceError::HeadNotDescendantOfFinalized);
        }
        if data.slot < data.head.slot {
            return Err(ForkChoiceError::AttestationSlotBeforeHead);
        }
        let admission = self.time + self.gossip_disparity_intervals;
        let max_slot = admission / self.intervals_per_slot;
        if data.slot.get() > max_slot {
            return Err(ForkChoiceError::AttestationTooFarInFuture);
        }
        Ok(())
    }
}

fn vote_is_newer(existing: &AttestationData, candidate: &AttestationData) -> bool {
    if candidate.slot > existing.slot {
        return true;
    }
    if candidate.slot < existing.slot {
        return false;
    }
    candidate.hash_tree_root() > existing.hash_tree_root()
}
