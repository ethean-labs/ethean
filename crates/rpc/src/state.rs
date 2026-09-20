//! Shared chain snapshot for the Lean HTTP listener.

use crate::dto::{FinalizedView, HeadView, SyncView};
use ethean_primitives::{Hash32, Slot, HASH32_ZERO};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Live values served by `/lean/v1/…` (updated by the node duty loop).
#[derive(Debug, Clone)]
pub struct ApiSnapshot {
    /// Network label (e.g. `pq-devnet-5`).
    pub network: String,
    /// libp2p peer id hex/base58, or empty when swarm is down.
    pub peer_id: String,
    pub head: HeadView,
    pub finalized: FinalizedView,
    pub sync: SyncView,
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
        }
    }
}

/// Process-wide Lean API state shared with the HTTP accept loop.
#[derive(Debug)]
pub struct SharedApiState {
    ready: AtomicBool,
    shutdown: AtomicBool,
    snap: Mutex<ApiSnapshot>,
    admin_token: String,
}

impl SharedApiState {
    /// Build with an optional admin bearer token (required on public binds).
    pub fn new(admin_token: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            ready: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            snap: Mutex::new(ApiSnapshot::default()),
            admin_token: admin_token.into(),
        })
    }

    /// Mirror metrics readiness into `/lean/v1/ready`.
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Relaxed);
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

    /// Replace the published chain views.
    pub fn publish(&self, snap: ApiSnapshot) {
        if let Ok(mut g) = self.snap.lock() {
            *g = snap;
        }
    }

    /// Clone the current snapshot.
    pub fn snapshot(&self) -> ApiSnapshot {
        self.snap
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Admin bearer expected by authorize helpers.
    pub fn admin_token(&self) -> &str {
        &self.admin_token
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
    }
}
