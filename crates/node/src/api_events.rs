//! Map chain events into the Lean HTTP admin event buffer.

use crate::events::ChainEvent;
use ethean_rpc::{AdminEvent, SharedApiState};
use ethean_validator::SuppressReason;
use std::sync::Arc;

/// Push redacted admin events derived from one duty / network step.
pub fn publish_chain_events(api: &SharedApiState, events: &[ChainEvent]) {
    for ev in events {
        match ev {
            ChainEvent::DutySuppressed { reason, .. } => {
                api.push_event(AdminEvent::DutySuppressed {
                    reason: suppress_label(*reason),
                });
            }
            ChainEvent::HeadUpdated { slot, .. } => {
                api.push_event(AdminEvent::HeadSlot { slot: *slot });
            }
            _ => {}
        }
    }
}

/// Helper when the client holds an optional API handle.
pub fn publish_optional(api: &Option<Arc<SharedApiState>>, events: &[ChainEvent]) {
    if let Some(api) = api {
        publish_chain_events(api, events);
    }
}

fn suppress_label(reason: SuppressReason) -> &'static str {
    match reason {
        SuppressReason::PreGenesis => "pre_genesis",
        SuppressReason::Syncing => "syncing",
        SuppressReason::MissingParentState => "missing_parent_state",
        SuppressReason::ProfileMismatch => "profile_mismatch",
        SuppressReason::SignerUnsafe => "signer_unsafe",
        SuppressReason::HeadLagExceeded => "head_lag_exceeded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::{Hash32, Slot};
    use ethean_validator::DutyTick;

    #[test]
    fn maps_suppressed_and_head() {
        let api = SharedApiState::new("");
        publish_chain_events(
            &api,
            &[
                ChainEvent::DutySuppressed {
                    tick: DutyTick {
                        slot: Slot::new(1),
                        interval: 0,
                        generation: 1,
                    },
                    reason: SuppressReason::Syncing,
                },
                ChainEvent::HeadUpdated {
                    root: Hash32::default(),
                    slot: 9,
                },
            ],
        );
        let drained = api.drain_events(8);
        assert!(drained
            .iter()
            .any(|e| matches!(e, AdminEvent::DutySuppressed { reason: "syncing" })));
        assert!(drained
            .iter()
            .any(|e| matches!(e, AdminEvent::HeadSlot { slot: 9 })));
    }
}
