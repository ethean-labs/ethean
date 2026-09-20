//! Assert leanSpec step `checks` and core `storeSnapshot` fields.

use crate::fc_runner::FcRunError;
use crate::hex::decode_hex_fixed;
use ethean_fork_choice::ForkChoiceStore;
use ethean_primitives::Hash32;
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) fn root_hex(root: &Hash32) -> String {
    let mut s = String::with_capacity(66);
    s.push_str("0x");
    for b in root {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub(crate) fn parse_root(s: &str) -> Result<Hash32, FcRunError> {
    decode_hex_fixed::<32>(s).map_err(|e| FcRunError::Step(format!("check root: {e}")))
}

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

fn head_slot(store: &ForkChoiceStore) -> Result<u64, FcRunError> {
    let block = store
        .blocks
        .get(&store.head)
        .ok_or_else(|| FcRunError::Step("checks headSlot: head block missing".into()))?;
    Ok(block.slot.get())
}

/// Validate optional `checks` object after a step.
pub fn apply_checks(store: &ForkChoiceStore, step: &Value) -> Result<(), FcRunError> {
    let Some(checks) = step.get("checks") else {
        return Ok(());
    };
    if let Some(want) = checks.get("time").and_then(|v| v.as_u64()) {
        if store.time != want {
            return Err(FcRunError::Step(format!(
                "checks.time got {}, want {want}",
                store.time
            )));
        }
    }
    if let Some(want) = checks.get("headSlot").and_then(|v| v.as_u64()) {
        let got = head_slot(store)?;
        if got != want {
            return Err(FcRunError::Step(format!(
                "checks.headSlot got {got}, want {want}"
            )));
        }
    }
    if let Some(want_s) = checks.get("headRoot").and_then(|v| v.as_str()) {
        let want = parse_root(want_s)?;
        if store.head != want {
            return Err(FcRunError::Step(format!(
                "checks.headRoot got {}, want {want_s}",
                root_hex(&store.head)
            )));
        }
    }
    if let Some(want) = checks.get("latestJustifiedSlot").and_then(|v| v.as_u64()) {
        let got = store.latest_justified.slot.get();
        if got != want {
            return Err(FcRunError::Step(format!(
                "checks.latestJustifiedSlot got {got}, want {want}"
            )));
        }
    }
    if let Some(want_s) = checks.get("latestJustifiedRoot").and_then(|v| v.as_str()) {
        let want = parse_root(want_s)?;
        if store.latest_justified.root != want {
            return Err(FcRunError::Step(format!(
                "checks.latestJustifiedRoot got {}, want {want_s}",
                root_hex(&store.latest_justified.root)
            )));
        }
    }
    if let Some(want) = checks.get("latestFinalizedSlot").and_then(|v| v.as_u64()) {
        let got = store.latest_finalized.slot.get();
        if got != want {
            return Err(FcRunError::Step(format!(
                "checks.latestFinalizedSlot got {got}, want {want}"
            )));
        }
    }
    if let Some(want_s) = checks.get("latestFinalizedRoot").and_then(|v| v.as_str()) {
        let want = parse_root(want_s)?;
        if store.latest_finalized.root != want {
            return Err(FcRunError::Step(format!(
                "checks.latestFinalizedRoot got {}, want {want_s}",
                root_hex(&store.latest_finalized.root)
            )));
        }
    }
    if let Some(want) = checks.get("safeTargetSlot").and_then(|v| v.as_u64()) {
        let got = store
            .blocks
            .get(&store.safe_target)
            .map(|b| b.slot.get())
            .ok_or_else(|| {
                FcRunError::Step("checks.safeTargetSlot: safe block missing".into())
            })?;
        if got != want {
            return Err(FcRunError::Step(format!(
                "checks.safeTargetSlot got {got}, want {want}"
            )));
        }
    }
    if let Some(want_s) = checks.get("safeTargetRoot").and_then(|v| v.as_str()) {
        let want = parse_root(want_s)?;
        if store.safe_target != want {
            return Err(FcRunError::Step(format!(
                "checks.safeTargetRoot got {}, want {want_s}",
                root_hex(&store.safe_target)
            )));
        }
    }
    Ok(())
}

/// True when leanSpec `StoreChecks` asserted a safe-target field.
fn checks_assert_safe_target(step: &Value) -> bool {
    step.get("checks").is_some_and(|c| {
        c.get("safeTargetSlot").is_some()
            || c.get("safeTargetRoot").is_some()
            || c.get("safeTargetRootLabel").is_some()
    })
}

/// Validate core fields of optional `storeSnapshot` (weights / pools deferred).
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

/// Run `checks` then core `storeSnapshot` when present on the step.
pub fn apply_step_assertions(store: &ForkChoiceStore, step: &Value) -> Result<(), FcRunError> {
    apply_checks(store, step)?;
    apply_store_snapshot(store, step)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn skips_when_no_assertions() {
        let step = json!({});
        assert!(step.get("checks").is_none());
        assert!(step.get("storeSnapshot").is_none());
    }
}
