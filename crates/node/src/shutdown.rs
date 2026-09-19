//! Graceful shutdown: stop new duties, preserve signer/storage boundaries.

/// Shutdown phase for the node duty loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPhase {
    /// Accepting new duties.
    Running,
    /// Stop scheduling; finish in-flight durable work.
    Draining,
    /// No further duty or import commands.
    Stopped,
}

/// Tracks shutdown progression.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownState {
    phase: ShutdownPhase,
}

impl ShutdownState {
    /// Begin draining: reject new duties, allow completions.
    pub fn begin_drain(&mut self) {
        if self.phase == ShutdownPhase::Running {
            self.phase = ShutdownPhase::Draining;
        }
    }

    /// Mark fully stopped after durable boundaries flush.
    pub fn finish(&mut self) {
        self.phase = ShutdownPhase::Stopped;
    }

    /// Current phase.
    pub fn phase(&self) -> ShutdownPhase {
        self.phase
    }

    /// True while new duties may start.
    pub fn accepts_new_duties(&self) -> bool {
        self.phase == ShutdownPhase::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drain_then_stop() {
        let mut s = ShutdownState::default();
        assert!(s.accepts_new_duties());
        s.begin_drain();
        assert!(!s.accepts_new_duties());
        s.finish();
        assert_eq!(s.phase(), ShutdownPhase::Stopped);
    }
}
