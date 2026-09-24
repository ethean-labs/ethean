//! Shared chain snapshot for the Lean HTTP listener.

use crate::dto::{DutiesView, FinalizedView, ForkChoiceStatsView, HeadView, SyncView};
use crate::events::{AdminEvent, EventBuffer};
use crate::test_driver::DriverHandle;
use crate::view::ForkChoiceView;
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Live values served by `/lean/v0` and `/lean/v1` (updated by the node duty loop).
#[derive(Debug, Clone)]
pub struct ApiSnapshot {
    /// Network label (e.g. `pq-devnet-5`).
    pub network: String,
    /// libp2p peer id hex/base58, or empty when swarm is down.
    pub peer_id: String,
    pub head: HeadView,
    pub finalized: FinalizedView,
    pub sync: SyncView,
    /// Fork-choice + finalized SSZ pair for hive `/lean/v0`.
    pub fork_choice: ForkChoiceView,
    /// Live-store stats for `/lean/v1/chain/fork_choice`.
    pub fork_choice_stats: ForkChoiceStatsView,
    /// Bounded local duty visibility for `/lean/v1/validator/duties`.
    pub duties: DutiesView,
    /// Mirror of `/lean/v1/ready` for identity JSON.
    pub ready: bool,
}

impl Default for ApiSnapshot {
    fn default() -> Self {
        Self {
            network: String::new(),
            peer_id: String::new(),
            head: HeadView {
                slot: Slot::new(0),
                root: HASH32_ZERO,
            },
            finalized: FinalizedView {
                slot: Slot::new(0),
                root: HASH32_ZERO,
                trust_source: "local".into(),
            },
            sync: SyncView {
                syncing: false,
                head_slot: Slot::new(0),
                peer_horizon_slot: Slot::new(0),
            },
            fork_choice: ForkChoiceView::default(),
            fork_choice_stats: ForkChoiceStatsView {
                live: false,
                head_root: HASH32_ZERO,
                safe_target_root: HASH32_ZERO,
                safe_target_slot: 0,
                justified_root: HASH32_ZERO,
                finalized_root: HASH32_ZERO,
                reorg_total: 0,
                blocks: 0,
                pending_votes: 0,
                known_votes: 0,
            },
            duties: DutiesView::default(),
            ready: false,
        }
    }
}

/// Process-wide Lean API state shared with the HTTP accept loop.
pub struct SharedApiState {
    ready: AtomicBool,
    shutdown: AtomicBool,
    aggregator: AtomicBool,
    snap: Mutex<ApiSnapshot>,
    events: Mutex<EventBuffer>,
    last_head_slot: Mutex<Option<u64>>,
    admin_token: String,
    driver: Mutex<Option<DriverHandle>>,
}

impl std::fmt::Debug for SharedApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedApiState")
            .field("ready", &self.is_ready())
            .field("driver", &self.driver().is_some())
            .finish()
    }
}

impl SharedApiState {
    /// Build with an optional admin bearer token (required on public binds).
    pub fn new(admin_token: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            ready: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            aggregator: AtomicBool::new(false),
            snap: Mutex::new(ApiSnapshot::default()),
            events: Mutex::new(EventBuffer::default()),
            last_head_slot: Mutex::new(None),
            admin_token: admin_token.into(),
            driver: Mutex::new(None),
        })
    }

    /// Mirror metrics readiness into `/lean/v1/ready`.
    pub fn set_ready(&self, ready: bool) {
        let prev = self.ready.swap(ready, Ordering::Relaxed);
        if prev != ready {
            self.push_event(AdminEvent::Readiness { ready });
        }
    }

    /// Current readiness bit.
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    /// Admin shutdown latch (polled by the node signal loop).
    pub fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// True after a successful admin shutdown POST.
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Seed aggregator role from CLI / hive `IS_AGGREGATOR`.
    pub fn set_aggregator(&self, enabled: bool) {
        self.aggregator.store(enabled, Ordering::Relaxed);
    }

    /// Current aggregator role (atomic; POST `/admin/aggregator` flips this).
    pub fn is_aggregator(&self) -> bool {
        self.aggregator.load(Ordering::Relaxed)
    }

    /// Swap the aggregator flag; returns the previous value.
    pub fn swap_aggregator(&self, enabled: bool) -> bool {
        self.aggregator.swap(enabled, Ordering::Relaxed)
    }

    /// Replace the published chain views; emit head-slot events on change.
    pub fn publish(&self, snap: ApiSnapshot) {
        let slot = snap.head.slot.get();
        if let Ok(mut last) = self.last_head_slot.lock() {
            if *last != Some(slot) {
                *last = Some(slot);
                drop(last);
                self.push_event(AdminEvent::HeadSlot { slot });
            }
        }
        if let Ok(mut g) = self.snap.lock() {
            *g = snap;
        }
    }

    /// Clone the current snapshot.
    pub fn snapshot(&self) -> ApiSnapshot {
        self.snap.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Push a redacted admin event into the ring buffer.
    pub fn push_event(&self, ev: AdminEvent) {
        if let Ok(mut q) = self.events.lock() {
            q.push(ev);
        }
    }

    /// Drain up to `n` buffered admin events (JSON poll consumer).
    pub fn drain_events(&self, n: usize) -> Vec<AdminEvent> {
        self.events
            .lock()
            .map(|mut q| q.drain(n))
            .unwrap_or_default()
    }

    /// Events still waiting after the last drain (or since the last push).
    pub fn events_pending(&self) -> usize {
        self.events.lock().map(|q| q.len()).unwrap_or(0)
    }

    /// Admin bearer expected by authorize helpers.
    pub fn admin_token(&self) -> &str {
        &self.admin_token
    }

    /// Enable the hive `test_driver` routes.
    pub fn install_driver(&self, driver: DriverHandle) {
        if let Ok(mut g) = self.driver.lock() {
            *g = Some(driver);
        }
    }

    /// Installed test driver, if any.
    pub fn driver(&self) -> Option<DriverHandle> {
        self.driver.lock().ok().and_then(|g| g.clone())
    }
}

/// Format a 32-byte root as lowercase hex (no `0x` prefix).
pub fn hex_root(root: &Hash32) -> String {
    let mut out = String::with_capacity(64);
    for b in root {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_round_trip() {
        let st = SharedApiState::new("");
        let mut snap = ApiSnapshot::default();
        snap.network = "local".into();
        snap.head.slot = Slot::new(7);
        st.publish(snap);
        assert_eq!(st.snapshot().head.slot.get(), 7);
        assert_eq!(st.snapshot().network, "local");
        let drained = st.drain_events(8);
        assert!(drained
            .iter()
            .any(|e| matches!(e, AdminEvent::HeadSlot { slot: 7 })));
    }

    #[test]
    fn aggregator_swap() {
        let st = SharedApiState::new("");
        assert!(!st.is_aggregator());
        assert!(!st.swap_aggregator(true));
        assert!(st.is_aggregator());
        assert!(st.swap_aggregator(false));
        assert!(!st.is_aggregator());
    }
}
