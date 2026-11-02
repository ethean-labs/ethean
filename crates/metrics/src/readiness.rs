//! Process readiness gates (false until all subsystems pass).

use crate::error::{MetricsError, Result};

/// Subsystem readiness bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readiness {
    pub storage: bool,
    pub crypto: bool,
    pub signer: bool,
    pub network: bool,
    pub prover: bool,
    /// Head slot moved within the stall window.
    pub head_progress: bool,
    /// Prover worker still running, or no worker expected.
    pub prover_alive: bool,
    /// Type-2 job has not occupied the prover past the stall window.
    pub block_proof_progress: bool,
}

impl Default for Readiness {
    fn default() -> Self {
        Self {
            storage: false,
            crypto: false,
            signer: false,
            network: false,
            prover: false,
            head_progress: true,
            prover_alive: true,
            block_proof_progress: true,
        }
    }
}

impl Readiness {
    /// True only when every gate is set.
    pub fn is_ready(&self) -> bool {
        self.storage
            && self.crypto
            && self.signer
            && self.network
            && self.prover
            && self.head_progress
            && self.prover_alive
            && self.block_proof_progress
    }

    /// Fail if not ready (for /readyz).
    pub fn require(&self) -> Result<()> {
        if self.is_ready() {
            Ok(())
        } else {
            Err(MetricsError::NotReady(format!("{self:?}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_gates_required() {
        let mut r = Readiness {
            storage: true,
            crypto: true,
            signer: true,
            network: true,
            prover: false,
            ..Readiness::default()
        };
        assert!(!r.is_ready());
        r.prover = true;
        assert!(r.require().is_ok());
        r.head_progress = false;
        assert!(!r.is_ready());
    }
}
