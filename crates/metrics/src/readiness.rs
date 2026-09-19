//! Process readiness gates (false until all subsystems pass).

use crate::error::{MetricsError, Result};

/// Subsystem readiness bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Readiness {
    pub storage: bool,
    pub crypto: bool,
    pub signer: bool,
    pub network: bool,
    pub prover: bool,
}

impl Readiness {
    /// True only when every gate is set.
    pub fn is_ready(&self) -> bool {
        self.storage && self.crypto && self.signer && self.network && self.prover
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
        };
        assert!(!r.is_ready());
        r.prover = true;
        assert!(r.require().is_ok());
    }
}
