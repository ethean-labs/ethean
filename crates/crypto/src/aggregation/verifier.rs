//! Aggregate-proof verifier interface.
//!
//! Consensus code builds the expected key sets and `(message, slot)` bindings
//! from state; an implementation (leanMultisig in `ethean-multisig`) checks the
//! proof against them. Keeping the trait here lets `ethean-transition` verify
//! blocks without linking the zkVM.

use crate::error::Result;
use crate::signature::PublicKey;

/// One component of a multi-message proof: who signed, what, and when.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofComponent {
    /// Keys of every signer of this component, in validator-index order.
    pub public_keys: Vec<PublicKey>,
    /// 32-byte signed message (an SSZ hash tree root).
    pub message: [u8; 32],
    /// Slot the signatures were produced for (the XMSS epoch).
    pub slot: u64,
}

/// Verifies leanMultisig single-message (Type-1) and multi-message (Type-2) proofs.
pub trait AggregateVerifier: Send + Sync {
    /// Check that every key in `public_keys` signed `message` at `slot`.
    fn verify_single(
        &self,
        proof: &[u8],
        public_keys: &[PublicKey],
        message: &[u8; 32],
        slot: u64,
    ) -> Result<()>;

    /// Check a merged proof against its ordered component bindings.
    fn verify_multi(&self, proof: &[u8], components: &[ProofComponent]) -> Result<()>;
}
