//! Structural state transition for decoded gossip blocks.

use crate::chain_owner::ChainOwner;
use crate::gossip_decode::DecodedBlockGossip;
use crate::shutdown::{ShutdownPhase, ShutdownState};
use ethean_primitives::Hash32;
use ethean_transition::{
    apply_block, apply_block_unverified, verify_proposer_signature, TransitionContext,
};
use ethean_crypto::ProductionBackend;

/// Outcome of attempting to import a decoded gossip block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GossipStfResult {
    /// Parent mismatched or shutdown; head unchanged.
    Skipped,
    /// No local state/profile; head root advanced only.
    RootOnly { root: Hash32 },
    /// `apply_block_unverified` updated head state and root.
    Applied { root: Hash32 },
    /// Verified `apply_block(SignedBlock)` updated head state and root.
    AppliedVerified { root: Hash32 },
    /// Local state present but transition failed; head unchanged.
    Rejected,
}

/// Import a decoded block: verified STF when a proof is present, else structural.
pub fn import_decoded_block(
    owner: &mut ChainOwner,
    shutdown: &ShutdownState,
    decoded: &DecodedBlockGossip,
) -> GossipStfResult {
    if shutdown.phase() == ShutdownPhase::Stopped {
        return GossipStfResult::Skipped;
    }
    if decoded.parent != owner.head_root {
        return GossipStfResult::Skipped;
    }

    let (Some(pre), Some(profile)) = (owner.head_state.clone(), owner.profile.clone()) else {
        owner.head_root = decoded.root;
        return GossipStfResult::RootOnly {
            root: decoded.root,
        };
    };

    let ctx = TransitionContext::new(profile);
    if let Some(sig) = &decoded.proposer_signature {
        // Claimed sidecar bindings fail closed against the registry proposal key.
        if verify_proposer_signature(&pre, &decoded.block, sig, &ProductionBackend).is_err() {
            return GossipStfResult::Rejected;
        }
    }
    if let Some(signed) = &decoded.signed {
        if !signed.proof.proof.is_empty() {
            return match apply_block(&pre, signed, &ctx) {
                Ok(out) => {
                    owner.head_state = Some(out.post_state);
                    owner.head_root = decoded.root;
                    GossipStfResult::AppliedVerified {
                        root: decoded.root,
                    }
                }
                Err(_) => GossipStfResult::Rejected,
            };
        }
    }

    match apply_block_unverified(&pre, &decoded.block, &ctx) {
        Ok(out) => {
            owner.head_state = Some(out.post_state);
            owner.head_root = decoded.root;
            GossipStfResult::Applied {
                root: decoded.root,
            }
        }
        Err(_) => GossipStfResult::Rejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex, HASH32_ZERO};
    use ethean_profile::lstar_devnet;
    use ethean_transition::process_slots;
    use ethean_types::{
        Block, BlockBody, BlockHeader, Checkpoint, GenesisConfig, State, Validator,
    };

    fn sample_state(validators: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..validators {
            vals.push(
                Validator::new(Bytes52::ZERO, Bytes52::ZERO, ValidatorIndex::new(i as u64))
                    .unwrap(),
            );
        }
        State {
            config: GenesisConfig::new(1_700_000_000),
            slot: Slot::ZERO,
            latest_block_header: BlockHeader::default(),
            latest_justified: Checkpoint::genesis(),
            latest_finalized: Checkpoint::genesis(),
            historical_block_hashes: Vec::new(),
            justified_slots: Vec::new(),
            validators: vals,
            justifications_roots: Vec::new(),
            justifications_validators: Vec::new(),
        }
    }

    #[test]
    fn applies_structural_transition_when_state_present() {
        let pre = sample_state(3);
        let mut advanced = pre.clone();
        process_slots(&mut advanced, Slot::new(1)).unwrap();
        let parent_root = advanced.latest_block_header.hash_tree_root();
        let mut block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(1),
            parent_root,
            state_root: HASH32_ZERO,
            body: BlockBody::default(),
        };
        let mut trial = pre.clone();
        process_slots(&mut trial, Slot::new(1)).unwrap();
        ethean_transition::process_block(
            &mut trial,
            &block,
            &TransitionContext::new(lstar_devnet().unwrap()),
        )
        .unwrap();
        block.state_root = trial.hash_tree_root().unwrap();
        let root = block.hash_tree_root().unwrap();

        let mut owner = ChainOwner::new(2);
        owner.head_root = parent_root;
        owner.head_state = Some(pre);
        owner.profile = Some(lstar_devnet().unwrap());
        let shutdown = ShutdownState::default();
        let decoded = DecodedBlockGossip {
            root,
            parent: parent_root,
            block,
            signed: None,
            proposer_signature: None,
        };
        assert_eq!(
            import_decoded_block(&mut owner, &shutdown, &decoded),
            GossipStfResult::Applied { root }
        );
        assert_eq!(owner.head_root, root);
        assert_eq!(owner.head_state.as_ref().unwrap().slot, Slot::new(1));
    }

    #[test]
    fn root_only_without_local_state() {
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        let shutdown = ShutdownState::default();
        let block = Block {
            slot: Slot::new(4),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [7u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let decoded = DecodedBlockGossip {
            root,
            parent: [1u8; 32],
            block,
            signed: None,
            proposer_signature: None,
        };
        assert_eq!(
            import_decoded_block(&mut owner, &shutdown, &decoded),
            GossipStfResult::RootOnly { root }
        );
        assert!(owner.head_state.is_none());
    }

    #[test]
    fn rejects_signed_block_with_bad_proof_when_state_present() {
        use ethean_types::{MultiMessageAggregate, SignedBlock};

        let pre = sample_state(2);
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        owner.head_state = Some(pre);
        owner.profile = Some(lstar_devnet().unwrap());
        let shutdown = ShutdownState::default();
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [7u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let signed = SignedBlock {
            block: block.clone(),
            proof: MultiMessageAggregate {
                proof: vec![1, 2, 3, 4],
            },
        };
        let decoded = DecodedBlockGossip {
            root,
            parent: [1u8; 32],
            block,
            signed: Some(signed),
            proposer_signature: None,
        };
        assert_eq!(
            import_decoded_block(&mut owner, &shutdown, &decoded),
            GossipStfResult::Rejected
        );
        assert_eq!(owner.head_root, [1u8; 32]);
    }

    #[test]
    fn rejects_bad_proposer_sidecar_when_state_present() {
        let pre = sample_state(2);
        let mut owner = ChainOwner::new(2);
        owner.head_root = [1u8; 32];
        owner.head_state = Some(pre);
        owner.profile = Some(lstar_devnet().unwrap());
        let shutdown = ShutdownState::default();
        let block = Block {
            slot: Slot::new(1),
            proposer_index: ValidatorIndex::new(0),
            parent_root: [1u8; 32],
            state_root: [7u8; 32],
            body: BlockBody::default(),
        };
        let root = block.hash_tree_root().unwrap();
        let decoded = DecodedBlockGossip {
            root,
            parent: [1u8; 32],
            block,
            signed: None,
            proposer_signature: Some(vec![0u8; ethean_crypto::SIGNATURE_BYTES]),
        };
        assert_eq!(
            import_decoded_block(&mut owner, &shutdown, &decoded),
            GossipStfResult::Rejected
        );
        assert_eq!(owner.head_root, [1u8; 32]);
    }
}
