//! What the node can do cryptographically, for boot logs and readiness.

use std::path::PathBuf;

use ethean_multisig::{ProverConfig, LEANVM_REV};

/// Crypto capabilities of this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoStatus {
    /// Native leanSpec XMSS signing and verification (always compiled in).
    pub xmss: bool,
    /// leanMultisig revision used for in-process proof verification.
    pub verifier_rev: &'static str,
    /// Prover binary used for aggregation and block proofs, when found.
    pub prover: Option<PathBuf>,
}

impl CryptoStatus {
    /// Probe the environment (`ETHEAN_PROVER_BIN` or a sibling `ethean-prover`).
    pub fn probe() -> Self {
        Self {
            xmss: true,
            verifier_rev: LEANVM_REV,
            prover: ProverConfig::discover().map(|c| c.binary),
        }
    }

    /// True when this node can produce aggregates and block proofs.
    pub fn can_prove(&self) -> bool {
        self.prover.is_some()
    }
}
