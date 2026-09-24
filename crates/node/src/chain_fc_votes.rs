//! Feed verified votes and aggregates into the live fork-choice store.

use crate::chain_owner::ChainOwner;
use ethean_primitives::ValidatorIndex;
use ethean_types::AttestationData;
use tracing::debug;

impl ChainOwner {
    /// Structural pending-vote ingest after signature verification.
    ///
    /// Head / safe-target move on later interval ticks (`accept_new_attestations`
    /// / `update_safe_target`), not on each gossip vote.
    pub fn fc_on_attestation(&mut self, validator: ValidatorIndex, data: AttestationData) {
        let Some(fc) = self.fc.as_mut() else {
            return;
        };
        if let Err(e) = fc.on_attestation_data(validator, data) {
            debug!(error = %e, "fork-choice on_attestation_data skipped");
            return;
        }
        self.sync_from_fork_choice();
    }

    /// Structural gossip aggregate ingest (participants → pending votes).
    pub fn fc_on_aggregated(&mut self, data: AttestationData, participants: &[bool]) {
        let Some(fc) = self.fc.as_mut() else {
            return;
        };
        if let Err(e) = fc.on_aggregated_attestation(data, participants) {
            debug!(error = %e, "fork-choice on_aggregated_attestation skipped");
            return;
        }
        self.sync_from_fork_choice();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_types::{
        AttestationData, BlockBody, BlockHeader, Checkpoint, GenesisConfig, State, Validator,
    };

    fn genesis_owner() -> ChainOwner {
        let val = Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(0)).unwrap();
        let mut st = State {
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
        // Genesis header keeps `state_root` zero (leanSpec); the anchor block fills it.
        st.latest_block_header.body_root = BlockBody::default().hash_tree_root().unwrap();
        let mut owner = ChainOwner::new(2);
        owner.profile = Some(lstar_devnet().unwrap());
        owner.head_state = Some(st);
        owner.head_root = owner
            .head_state
            .as_ref()
            .unwrap()
            .latest_block_header
            .hash_tree_root();
        owner.try_init_fork_choice();
        owner
    }

    #[test]
    fn vote_lands_in_pending_pool() {
        let mut owner = genesis_owner();
        assert!(owner.fc.is_some());
        let root = owner.head_root;
        let cp = Checkpoint {
            root,
            slot: Slot::ZERO,
        };
        let data = AttestationData {
            slot: Slot::ZERO,
            head: cp,
            target: cp,
            source: cp,
        };
        owner.fc_on_attestation(ValidatorIndex::new(0), data);
        let fc = owner.fc.as_ref().unwrap();
        assert!(fc.latest_new_attestations.contains_key(&ValidatorIndex::new(0)));
        let _ = HASH32_ZERO;
    }
}
