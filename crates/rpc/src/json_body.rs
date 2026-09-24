//! JSON response bodies for Lean HTTP routes (manual `serde_json`, no Beacon fields).

use crate::dto::{FinalizedView, ForkChoiceView, HeadView, SyncView};
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

/// Encode a drained admin event backlog (JSON poll; not a long-lived SSE body).
pub fn events_json(events: &[crate::events::AdminEvent]) -> Value {
    use crate::events::AdminEvent;
    let items: Vec<Value> = events
        .iter()
        .map(|e| match e {
            AdminEvent::DutySuppressed { reason } => json!({
                "kind": "duty_suppressed",
                "reason": reason,
            }),
            AdminEvent::HeadSlot { slot } => json!({
                "kind": "head_slot",
                "slot": slot,
            }),
            AdminEvent::Readiness { ready } => json!({
                "kind": "readiness",
                "ready": ready,
            }),
        })
        .collect();
    json!({ "events": items })
}

/// Encode fork-choice operator view.
pub fn fork_choice_json(v: &ForkChoiceView) -> Value {
    json!({
        "live": v.live,
        "head_root": hex_root(&v.head_root),
        "safe_target_root": hex_root(&v.safe_target_root),
        "safe_target_slot": v.safe_target_slot,
        "justified_root": hex_root(&v.justified_root),
        "finalized_root": hex_root(&v.finalized_root),
        "reorg_total": v.reorg_total,
        "blocks": v.blocks,
        "pending_votes": v.pending_votes,
        "known_votes": v.known_votes,
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

    #[test]
    fn fork_choice_reports_live_flag() {
        let v = ForkChoiceView {
            live: true,
            head_root: HASH32_ZERO,
            safe_target_root: [1u8; 32],
            safe_target_slot: 4,
            justified_root: HASH32_ZERO,
            finalized_root: HASH32_ZERO,
            reorg_total: 2,
            blocks: 3,
            pending_votes: 1,
            known_votes: 5,
        };
        let j = fork_choice_json(&v);
        assert_eq!(j["live"], true);
        assert_eq!(j["safe_target_slot"], 4);
        assert_eq!(j["reorg_total"], 2);
    }
}
