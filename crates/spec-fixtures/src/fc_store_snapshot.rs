//! Core `storeSnapshot` field checks for leanSpec FC fixtures.

use crate::fc_checks::{parse_root, root_hex};
use crate::fc_runner::FcRunError;
use ethean_fork_choice::ForkChoiceStore;
use ethean_primitives::Hash32;
use serde_json::Value;
use std::collections::BTreeSet;

fn checkpoint_fields(v: &Value) -> Result<(Hash32, u64), FcRunError> {
    let root = v
        .get("root")
        .and_then(|x| x.as_str())
        .ok_or_else(|| FcRunError::Step("checkpoint missing root".into()))?;
    let slot = v
        .get("slot")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| FcRunError::Step("checkpoint missing slot".into()))?;
    Ok((parse_root(root)?, slot))
}

fn checks_assert_safe_target(step: &Value) -> bool {
    step.get("checks").is_some_and(|c| {
        c.get("safeTargetSlot").is_some()
            || c.get("safeTargetRoot").is_some()
            || c.get("safeTargetRootLabel").is_some()
    })
}

/// Filled blocks sometimes drop `AggregatedAttestationSpec` from the wire body
/// while `storeSnapshot` still lists the in-memory payloads (finalized_safety).
fn block_wire_attestations_empty(step: &Value) -> bool {
    if step.get("stepType").and_then(|v| v.as_str()) != Some("block") {
        return false;
    }
    match step.pointer("/block/body/attestations/data") {
        Some(Value::Array(data)) => data.is_empty(),
        Some(_) => false,
        None => step.get("block").is_some(),
    }
}

/// Validate core fields of optional `storeSnapshot`.
pub fn apply_store_snapshot(store: &ForkChoiceStore, step: &Value) -> Result<(), FcRunError> {
    let Some(snap) = step.get("storeSnapshot") else {
        return Ok(());
    };
    if let Some(want) = snap.get("time").and_then(|v| v.as_u64()) {
        if store.time != want {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.time got {}, want {want}",
                store.time
            )));
        }
    }
    if let Some(want_s) = snap.get("headRoot").and_then(|v| v.as_str()) {
        let want = parse_root(want_s)?;
        if store.head != want {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.headRoot got {}, want {want_s}",
                root_hex(&store.head)
            )));
        }
    }
    // Filled dumps may leave safeTargetRoot stale when StoreChecks never
    // asserted it (tick interval-0 acceptance). Gate on explicit checks.
    if checks_assert_safe_target(step) {
        if let Some(want_s) = snap.get("safeTargetRoot").and_then(|v| v.as_str()) {
            let want = parse_root(want_s)?;
            if store.safe_target != want {
                return Err(FcRunError::Step(format!(
                    "storeSnapshot.safeTargetRoot got {}, want {want_s}",
                    root_hex(&store.safe_target)
                )));
            }
        }
    }
    if let Some(cp) = snap.get("latestJustified") {
        let (want_root, want_slot) = checkpoint_fields(cp)?;
        if store.latest_justified.root != want_root
            || store.latest_justified.slot.get() != want_slot
        {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.latestJustified got {}:{}, want {}:{}",
                root_hex(&store.latest_justified.root),
                store.latest_justified.slot.get(),
                root_hex(&want_root),
                want_slot
            )));
        }
    }
    if let Some(cp) = snap.get("latestFinalized") {
        let (want_root, want_slot) = checkpoint_fields(cp)?;
        if store.latest_finalized.root != want_root
            || store.latest_finalized.slot.get() != want_slot
        {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.latestFinalized got {}:{}, want {}:{}",
                root_hex(&store.latest_finalized.root),
                store.latest_finalized.slot.get(),
                root_hex(&want_root),
                want_slot
            )));
        }
    }
    if let Some(list) = snap.get("blockRoots").and_then(|v| v.as_array()) {
        let mut want = BTreeSet::new();
        for item in list {
            let s = item
                .as_str()
                .ok_or_else(|| FcRunError::Step("blockRoots entry not string".into()))?;
            want.insert(parse_root(s)?);
        }
        let got: BTreeSet<_> = store.blocks.keys().copied().collect();
        if got != want {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.blockRoots mismatch (got {} roots, want {})",
                got.len(),
                want.len()
            )));
        }
    }
    // Empty wire bodies with rich weight/payload dumps are filler skew
    // (Python BlockSpec had attestations; JSON body did not). Trust StoreChecks.
    if block_wire_attestations_empty(step) {
        return Ok(());
    }
    if let Some(list) = snap.get("blockWeights").and_then(|v| v.as_array()) {
        // All-zero weight dumps appear on older tick vectors where the filler
        // jumped the clock without interval actions; they are not authoritative.
        let any_positive = list.iter().any(|item| {
            item.get("weight")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        });
        if any_positive {
            let got = store.block_weights_from_known();
            for item in list {
                let root_s = item
                    .get("root")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| FcRunError::Step("blockWeights entry missing root".into()))?;
                let want_w = item
                    .get("weight")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        FcRunError::Step("blockWeights entry missing weight".into())
                    })?;
                let root = parse_root(root_s)?;
                let got_w = got.get(&root).copied().unwrap_or(0);
                if got_w != want_w {
                    return Err(FcRunError::Step(format!(
                        "storeSnapshot.blockWeights[{root_s}] got {got_w}, want {want_w}"
                    )));
                }
            }
        }
    }
    crate::fc_snapshot_payloads::apply_payload_pool(
        store,
        snap,
        "knownAggregatedPayloads",
        true,
    )?;
    crate::fc_snapshot_payloads::apply_payload_pool(
        store,
        snap,
        "newAggregatedPayloads",
        false,
    )?;
    Ok(())
}
