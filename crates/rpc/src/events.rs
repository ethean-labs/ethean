//! Bounded admin event backlog with redaction.

use crate::limits::MAX_EVENT_BACKLOG;
use std::collections::VecDeque;

/// Redacted event kinds safe for privileged streams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminEvent {
    /// Duty suppressed (reason code only — no roots).
    DutySuppressed { reason: &'static str },
    /// Head advanced (slot only).
    HeadSlot { slot: u64 },
    /// Readiness flipped.
    Readiness { ready: bool },
}

/// Ring buffer of admin events.
#[derive(Debug, Default)]
pub struct EventBuffer {
    q: VecDeque<AdminEvent>,
}

impl EventBuffer {
    /// Push, dropping oldest when over capacity.
    pub fn push(&mut self, ev: AdminEvent) {
        if self.q.len() >= MAX_EVENT_BACKLOG {
            self.q.pop_front();
        }
        self.q.push_back(ev);
    }

    /// Drain up to `n` events.
    pub fn drain(&mut self, n: usize) -> Vec<AdminEvent> {
        let mut out = Vec::new();
        for _ in 0..n {
            match self.q.pop_front() {
                Some(e) => out.push(e),
                None => break,
            }
        }
        out
    }

    /// Current depth.
    pub fn len(&self) -> usize {
        self.q.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.q.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_oldest() {
        let mut b = EventBuffer::default();
        for i in 0..(MAX_EVENT_BACKLOG + 5) {
            b.push(AdminEvent::HeadSlot { slot: i as u64 });
        }
        assert_eq!(b.len(), MAX_EVENT_BACKLOG);
    }
}
