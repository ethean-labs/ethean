//! Build [`ethean_rpc::ForkChoiceView`] from [`ChainOwner`] for the Lean HTTP API.

use crate::api_ssz::{canonical_state, canonical_state_ssz, genesis_signed_pair, signed_block_ssz};
use crate::chain_owner::ChainOwner;
use ethean_primitives::{Hash32, HASH32_ZERO};
use ethean_rpc::{CheckpointBody, ForkChoiceBody, ForkChoiceNodeBody, ForkChoiceView};
use ethean_types::{SignedBlock, State};

/// Snapshot the owner's chain into the hive `/lean/v0` view.
///
/// Prefer a live [`ethean_fork_choice::ForkChoiceStore`] for node list + weights.
/// If the store is not wired, list durable / genesis blocks with weight 0
/// (weights are not faked).
pub fn fork_choice_view(owner: &ChainOwner) -> ForkChoiceView {
    if let Some(store) = owner.fc.as_ref() {
        return from_store(owner, store);
    }
    from_owner_blocks(owner)
}

fn from_store(owner: &ChainOwner, store: &ethean_fork_choice::ForkChoiceStore) -> ForkChoiceView {
    let snap = store.api_snapshot();
    let validator_count = if snap.validator_count > 0 {
        snap.validator_count
    } else {
        owner
            .head_state
            .as_ref()
            .map(|s| s.validators.len() as u64)
            .unwrap_or(0)
    };
    let nodes: Vec<ForkChoiceNodeBody> = snap
        .nodes
        .into_iter()
        .map(|n| ForkChoiceNodeBody {
            root: n.root,
            slot: n.slot,
            parent_root: n.parent_root,
            proposer_index: n.proposer_index,
            weight: n.weight,
        })
        .collect();
    let (state_ssz, block_ssz) = finalized_pair(
        owner,
        snap.finalized.root,
        snap.finalized.slot.get(),
    );
    ForkChoiceView {
        fork_choice: ForkChoiceBody {
            nodes,
            head: snap.head,
            justified: CheckpointBody {
                slot: snap.justified.slot.get(),
                root: snap.justified.root,
            },
            finalized: CheckpointBody {
                slot: snap.finalized.slot.get(),
                root: snap.finalized.root,
            },
            safe_target: snap.safe_target,
            validator_count,
        },
        finalized_state_ssz: state_ssz,
        finalized_block_ssz: block_ssz,
        chain_ready: true,
    }
}

fn from_owner_blocks(owner: &ChainOwner) -> ForkChoiceView {
    let Some(head_state) = owner.head_state.as_ref() else {
        return ForkChoiceView::default();
    };
    let validator_count = head_state.validators.len() as u64;
    if validator_count == 0 {
        return ForkChoiceView::default();
    }

    let mut nodes = Vec::new();
    for (root, payload) in &owner.durable_blocks {
        if let Ok(signed) = SignedBlock::ssz_decode(payload) {
            nodes.push(ForkChoiceNodeBody {
                root: *root,
                slot: signed.block.slot.get(),
                parent_root: signed.block.parent_root,
                proposer_index: signed.block.proposer_index.get(),
                weight: 0,
            });
        }
    }

    let genesis_pair = genesis_signed_pair(head_state).ok();
    if nodes.is_empty() || !nodes.iter().any(|n| n.slot == 0) {
        if let Some((signed, _canonical, genesis_root)) = genesis_pair.as_ref() {
            let root = *genesis_root;
            if !nodes.iter().any(|n| n.root == root) {
                nodes.insert(
                    0,
                    ForkChoiceNodeBody {
                        root,
                        slot: signed.block.slot.get(),
                        parent_root: signed.block.parent_root,
                        proposer_index: signed.block.proposer_index.get(),
                        weight: 0,
                    },
                );
            }
        }
    }

    let genesis_root = genesis_pair.as_ref().map(|(_, _, r)| *r);
    let at_genesis = head_state.slot.get() == 0
        && head_state.latest_finalized.slot.get() == 0
        && head_state.latest_justified.slot.get() == 0;

    let head = if at_genesis {
        genesis_root.unwrap_or(owner.head_root)
    } else if owner.head_root != HASH32_ZERO {
        owner.head_root
    } else {
        genesis_root.unwrap_or(HASH32_ZERO)
    };

    let finalized = if at_genesis {
        CheckpointBody {
            slot: 0,
            root: head,
        }
    } else if head_state.latest_finalized.root == HASH32_ZERO {
        CheckpointBody {
            slot: head_state.latest_finalized.slot.get(),
            root: head,
        }
    } else {
        CheckpointBody {
            slot: head_state.latest_finalized.slot.get(),
            root: head_state.latest_finalized.root,
        }
    };
    let justified = if at_genesis {
        CheckpointBody {
            slot: 0,
            root: head,
        }
    } else if head_state.latest_justified.root == HASH32_ZERO {
        CheckpointBody {
            slot: head_state.latest_justified.slot.get(),
            root: head,
        }
    } else {
        CheckpointBody {
            slot: head_state.latest_justified.slot.get(),
            root: head_state.latest_justified.root,
        }
    };

    let finalized_slot = finalized.slot;
    nodes.retain(|n| n.slot >= finalized_slot);
    nodes.sort_by_key(|n| n.slot);

    let safe_target = if at_genesis {
        head
    } else if owner.safe_target == HASH32_ZERO {
        head
    } else {
        owner.safe_target
    };

    let (state_ssz, block_ssz) = finalized_pair(owner, finalized.root, finalized_slot);

    ForkChoiceView {
        fork_choice: ForkChoiceBody {
            nodes,
            head,
            justified,
            finalized,
            safe_target,
            validator_count,
        },
        finalized_state_ssz: state_ssz,
        finalized_block_ssz: block_ssz,
        chain_ready: head != HASH32_ZERO || at_genesis,
    }
}

fn finalized_pair(
    owner: &ChainOwner,
    finalized_root: Hash32,
    finalized_slot: u64,
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let state_ssz = finalized_state_ssz(owner, finalized_root, finalized_slot);
    let block_ssz = finalized_block_ssz(owner, finalized_root, finalized_slot);
    (state_ssz, block_ssz)
}

fn finalized_state_ssz(
    owner: &ChainOwner,
    finalized_root: Hash32,
    finalized_slot: u64,
) -> Option<Vec<u8>> {
    if let Some(store) = owner.fc.as_ref() {
        if let Some(st) = store.block_states.get(&finalized_root) {
            return canonical_state_ssz(st).ok();
        }
    }
    let head = owner.head_state.as_ref()?;
    if head.slot.get() == finalized_slot || finalized_slot == 0 {
        return canonical_state_ssz(head).ok();
    }
    None
}

fn finalized_block_ssz(
    owner: &ChainOwner,
    finalized_root: Hash32,
    finalized_slot: u64,
) -> Option<Vec<u8>> {
    if let Some(bytes) = owner.durable_block(&finalized_root) {
        return Some(bytes.to_vec());
    }
    if let Some(store) = owner.fc.as_ref() {
        if let Some(block) = store.blocks.get(&finalized_root) {
            let signed = SignedBlock::new(block.clone(), Default::default());
            return signed_block_ssz(&signed).ok();
        }
    }
    if finalized_slot == 0 {
        let state = owner.head_state.as_ref()?;
        let (signed, _canonical, _root) = genesis_signed_pair(state).ok()?;
        return signed_block_ssz(&signed).ok();
    }
    None
}

/// Hive invariants for a fresh-node view.
pub fn fresh_node_ok(view: &ForkChoiceView) -> bool {
    let fc = &view.fork_choice;
    fc.nodes.len() == 1
        && fc.nodes[0].slot == 0
        && fc.nodes[0].parent_root == HASH32_ZERO
        && fc.nodes[0].proposer_index == 0
        && fc.nodes[0].weight == 0
        && fc.head == fc.nodes[0].root
        && fc.justified.slot == 0
        && fc.justified.root == fc.head
        && fc.finalized.slot == 0
        && fc.finalized.root == fc.head
        && fc.safe_target == fc.head
        && fc.validator_count > 0
        && view.finalized_state_ssz.as_ref().is_some_and(|b| !b.is_empty())
        && view.finalized_block_ssz.as_ref().is_some_and(|b| !b.is_empty())
}

/// Decode the published SSZ pair and check hive pairing invariants.
pub fn ssz_pair_ok(view: &ForkChoiceView) -> bool {
    let (Some(state_bytes), Some(block_bytes)) =
        (&view.finalized_state_ssz, &view.finalized_block_ssz)
    else {
        return false;
    };
    let Ok(state) = State::ssz_decode(state_bytes) else {
        return false;
    };
    let Ok(signed) = SignedBlock::ssz_decode(block_bytes) else {
        return false;
    };
    let Ok(state_root) = canonical_state(&state).hash_tree_root() else {
        return false;
    };
    let Ok(block_root) = signed.block.hash_tree_root() else {
        return false;
    };
    signed.block.slot == state.slot
        && signed.block.state_root == state_root
        && block_root == view.fork_choice.finalized.root
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_finality::seal_genesis_head;
    use ethean_genesis::local_smoke_genesis;

    #[test]
    fn fresh_owner_matches_hive_invariants() {
        let built = local_smoke_genesis(1_700_000_000).unwrap();
        let mut owner = ChainOwner::default();
        owner.head_state = Some(built.state);
        seal_genesis_head(&mut owner);
        let view = fork_choice_view(&owner);
        assert!(fresh_node_ok(&view), "fresh forkchoice: {:?}", view.fork_choice);
        assert!(ssz_pair_ok(&view), "ssz pair must match hive pairing");
    }
}
