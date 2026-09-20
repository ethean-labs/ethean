//! JSON response bodies for Lean HTTP routes (manual `serde_json`, no Beacon fields).

use crate::dto::{FinalizedView, HeadView, SyncView};
use crate::state::{hex_root, ApiSnapshot};
use serde_json::{json, Value};

/// Encode head view.
pub fn head_json(v: &HeadView) -> Value {
    json!({
        "slot": v.slot.get(),
        "root": hex_root(&v.root),
    })
}

/// Encode finalized view with explicit trust label.
pub fn finalized_json(v: &FinalizedView) -> Value {
    json!({
        "slot": v.slot.get(),
        "root": hex_root(&v.root),
        "trust_source": v.trust_source,
    })
}

/// Encode sync gate view.
pub fn sync_json(v: &SyncView) -> Value {
    json!({
        "syncing": v.syncing,
        "head_slot": v.head_slot.get(),
        "peer_horizon_slot": v.peer_horizon_slot.get(),
    })
}

/// Encode node identity from the shared snapshot.
pub fn identity_json(snap: &ApiSnapshot) -> Value {
    json!({
        "network": snap.network,
        "peer_id": snap.peer_id,
    })
}

/// Bounded duties placeholder (empty until validator schedule is wired).
pub fn duties_json() -> Value {
    json!({ "duties": [] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Slot, HASH32_ZERO};

    #[test]
    fn head_has_slot_and_root() {
        let v = HeadView {
            slot: Slot::new(3),
            root: HASH32_ZERO,
        };
        let j = head_json(&v);
        assert_eq!(j["slot"], 3);
        assert_eq!(j["root"].as_str().unwrap().len(), 64);
    }
}
