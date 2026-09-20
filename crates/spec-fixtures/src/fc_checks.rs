//! Assert leanSpec step `checks` and dispatch `storeSnapshot` checks.

use crate::fc_runner::FcRunError;
use crate::fc_store_snapshot::apply_store_snapshot;
use crate::hex::decode_hex_fixed;
use ethean_fork_choice::ForkChoiceStore;
use ethean_primitives::Hash32;
use serde_json::Value;

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
