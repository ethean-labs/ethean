//! Aggregate proof API — deferred to Phase 08.

use crate::error::CryptoError;

/// Placeholder aggregate type; construction is Phase 08.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregateProofDeferred;

impl AggregateProofDeferred {
    /// Always fails closed; no fake aggregate acceptance.
    pub fn verify_stub(_bytes: &[u8]) -> Result<(), CryptoError> {
        Err(CryptoError::AggregateDeferred)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_fails_closed() {
        assert_eq!(
            AggregateProofDeferred::verify_stub(&[0u8; 8]),
            Err(CryptoError::AggregateDeferred)
        );
    }
}
