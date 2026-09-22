//! Local finality helpers: self-apply proposals and full-registry attestations.

use crate::block_builder::PlanTransition;
use crate::chain_owner::ChainOwner;
use ethean_primitives::{Hash32, HASH32_ZERO};
use ethean_transition::{apply_block_unverified, TransitionContext};
use ethean_types::{
    AggregatedAttestation, AggregationBits, AttestationData, BlockBody, Checkpoint, State,
};
use tracing::{debug, info, warn};

/// Set `head_root` from the genesis header root when still zero.
///
/// Caches `latest_block_header.state_root` first (leanSpec `process_slots` rule)
/// so the first proposal's parent root matches the post-slot header hash.
pub fn seal_genesis_head(owner: &mut ChainOwner) {
    if owner.head_root != HASH32_ZERO {
        return;
    }
    let Some(state) = owner.head_state.as_mut() else {
        return;
    };
    if state.latest_block_header.state_root == HASH32_ZERO {
        if let Ok(root) = state.hash_tree_root() {
            state.latest_block_header.state_root = root;
        }
    }
    owner.head_root = state.latest_block_header.hash_tree_root();
    owner.refresh_fc_view();
    info!(
        head_root = %hex32(&owner.head_root),
        "sealed genesis head_root from latest_block_header"
    );
}

/// Build a full-registry aggregated attestation for 3SF-mini local smoke.
pub fn full_registry_attestation(state: &State, head: Checkpoint) -> Option<AggregatedAttestation> {
    let n = state.validators.len();
    if n == 0 {
        return None;
    }
    let source = state.latest_justified;
    let target = if head.slot > source.slot {
        head
    } else {
        source
    };
    let bits = AggregationBits::new(vec![true; n]).ok()?;
    Some(AggregatedAttestation {
        aggregation_bits: bits,
        data: AttestationData {
            slot: target.slot,
            head,
            target,
            source,
        },
    })
}

/// Apply a planned proposal to the local head (proposer self-import).
pub fn apply_planned_locally(
    owner: &mut ChainOwner,
    plan: &PlanTransition,
) -> Result<Hash32, String> {
    let (pre, profile) = match (owner.head_state.clone(), owner.profile.clone()) {
        (Some(s), Some(p)) => (s, p),
        _ => return Err("missing head state or profile".into()),
    };
    let ctx = TransitionContext::new(profile);
    let out = apply_block_unverified(&pre, &plan.block, &ctx).map_err(|e| e.to_string())?;
    let root = plan.block_root()?;
    let parent = plan.block.parent_root;
    owner.head_state = Some(out.post_state);
    owner.advance_head(root, parent);
    Ok(root)
}

/// After a successful local self-apply, advance justified/finalized with a
/// one-slot lag so Grafana panels climb without a multi-peer mesh.
///
/// This is a **local smoke** shortcut (not production 3SF-mini networking).
pub fn promote_local_checkpoints(owner: &mut ChainOwner, applied_root: Hash32) {
    let Some(state) = owner.head_state.as_mut() else {
        return;
    };
    let head = Checkpoint {
        root: applied_root,
        slot: state.slot,
    };
    if head.slot.get() <= state.latest_justified.slot.get() {
        return;
    }
    let prev_just = state.latest_justified;
    if prev_just.slot.get() > state.latest_finalized.slot.get() {
        state.latest_finalized = prev_just;
    }
    state.latest_justified = head;
    info!(
        justified = state.latest_justified.slot.get(),
        finalized = state.latest_finalized.slot.get(),
        "local finality promoted checkpoints"
    );
    // Mutating justified/finalized changes the state root; reseal so the next
    // proposal's parent_root matches `hash_tree_root(latest_block_header)`.
    if let Ok(sr) = state.hash_tree_root() {
        state.latest_block_header.state_root = sr;
    }
    owner.head_root = state.latest_block_header.hash_tree_root();
    owner.refresh_fc_view();
}

/// When aggregating locally, attach a full-registry vote to the planned body.
pub fn inject_local_aggregate(owner: &mut ChainOwner, plan: &mut PlanTransition) {
    let Some(state) = owner.head_state.as_ref() else {
        return;
    };
    let parent = Checkpoint {
        root: owner.head_root,
        slot: state.latest_block_header.slot,
    };
    let Some(agg) = full_registry_attestation(state, parent) else {
        return;
    };
    match BlockBody::new(vec![agg]) {
        Ok(body) => {
            plan.block.body = body;
            debug!("injected full-registry attestation for local finality");
        }
        Err(e) => warn!(error = %e, "could not inject local attestation body"),
    }
}

/// Recompute state_root after body mutation (must re-run trial transition).
pub fn rebind_plan_state_root(owner: &ChainOwner, plan: &mut PlanTransition) -> Result<(), String> {
    let (pre, profile) = match (owner.head_state.as_ref(), owner.profile.as_ref()) {
        (Some(s), Some(p)) => (s.clone(), p.clone()),
        _ => return Err("missing head state or profile".into()),
    };
    use ethean_transition::{process_block, process_slots};
    let ctx = TransitionContext::new(profile);
    let mut trial = pre;
    process_slots(&mut trial, plan.block.slot).map_err(|e| e.to_string())?;
    process_block(&mut trial, &plan.block, &ctx).map_err(|e| e.to_string())?;
    plan.block.state_root = trial.hash_tree_root().map_err(|e| e.to_string())?;
    plan.aggregate_proof.clear();
    plan.attestation_proofs.clear();
    Ok(())
}

fn hex32(root: &Hash32) -> String {
    root.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_builder::plan_from_pool;
    use crate::aggregation::AggregatePool;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_profile::lstar_devnet;
    use ethean_types::{BlockHeader, GenesisConfig, Validator};

    fn state_n(n: usize) -> State {
        let mut vals = Vec::new();
        for i in 0..n {
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
    fn self_apply_advances_head_slot() {
        let profile = lstar_devnet().unwrap();
        let pre = state_n(4);
        let mut owner = ChainOwner::new(4);
        owner.head_state = Some(pre);
        owner.profile = Some(profile.clone());
        seal_genesis_head(&mut owner);
        let parent = owner.head_root;
        let pool = AggregatePool::default();
        let mut plan = plan_from_pool(
            &pool,
            parent,
            Slot::new(1),
            ValidatorIndex::new(1),
            owner.head_state.as_ref().unwrap(),
            profile,
            16,
        )
        .unwrap();
        inject_local_aggregate(&mut owner, &mut plan);
        rebind_plan_state_root(&owner, &mut plan).unwrap();
        let root = apply_planned_locally(&mut owner, &plan).unwrap();
        assert_ne!(root, HASH32_ZERO);
        assert_eq!(owner.head_state.as_ref().unwrap().slot.get(), 1);
        promote_local_checkpoints(&mut owner, root);
        assert!(owner.head_state.as_ref().unwrap().latest_justified.slot.get() >= 1);
    }
}
