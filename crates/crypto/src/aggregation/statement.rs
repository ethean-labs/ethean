//! Canonical aggregate statements (consensus-derived public inputs).

use crate::error::{CryptoError, Result};
use crate::hash::domain_digest;
use crate::aggregation::bindings::MAX_PROOF_BYTES;

/// Registry-sized participant cap (leanSpec VALIDATOR_REGISTRY_LIMIT).
pub const MAX_PARTICIPANTS: usize = 4096;

/// Proof generation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProofKind {
    /// One message, many participants (Type-1).
    Type1,
    /// Ordered merge of Type-1 components including proposer (Type-2).
    Type2,
}

/// Ordered unique validator indices covered by a proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantSet {
    indices: Vec<u32>,
}

impl ParticipantSet {
    /// Build from ordered unique indices; rejects duplicates and out-of-range.
    pub fn try_from_ordered(indices: Vec<u32>) -> Result<Self> {
        if indices.len() > MAX_PARTICIPANTS {
            return Err(CryptoError::InvalidAggregate(
                "too many participants".into(),
            ));
        }
        for (i, &idx) in indices.iter().enumerate() {
            if idx as usize >= MAX_PARTICIPANTS {
                return Err(CryptoError::InvalidAggregate(format!(
                    "participant index {idx} out of range"
                )));
            }
            if i > 0 && indices[i - 1] >= idx {
                return Err(CryptoError::InvalidAggregate(
                    "participants must be strictly increasing".into(),
                ));
            }
        }
        Ok(Self { indices })
    }

    /// Empty set (invalid for a finished proof, but usable while building).
    pub fn empty() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Borrow ordered indices.
    pub fn as_slice(&self) -> &[u32] {
        &self.indices
    }

    /// True when no participants.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Reference to a Type-1 child inside a Type-2 merge (message root only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type2ComponentRef {
    /// Message root of the child Type-1 statement.
    pub message_root: [u8; 32],
    /// Slot of the child Type-1 statement.
    pub slot: u64,
}

/// Consensus-derived public inputs for verify/prove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregateStatement {
    /// Proof kind.
    pub kind: ProofKind,
    /// Domain profile fingerprint bytes (caller-supplied, typically aggregation fingerprint hash).
    pub profile_digest: [u8; 32],
    /// Shared message root (Type-1) or block binding root (Type-2).
    pub message_root: [u8; 32],
    /// Slot every signer / component is bound to.
    pub slot: u64,
    /// Ordered unique participants (Type-1) or union coverage (Type-2).
    pub participants: ParticipantSet,
    /// Type-2 ordered component descriptors (empty for Type-1).
    pub components: Vec<Type2ComponentRef>,
}

impl AggregateStatement {
    /// Validate shape before FFI / backend.
    pub fn validate_shape(&self) -> Result<()> {
        match self.kind {
            ProofKind::Type1 => {
                if self.participants.is_empty() {
                    return Err(CryptoError::InvalidAggregate(
                        "Type-1 requires at least one participant".into(),
                    ));
                }
                if !self.components.is_empty() {
                    return Err(CryptoError::InvalidAggregate(
                        "Type-1 must not carry Type-2 components".into(),
                    ));
                }
            }
            ProofKind::Type2 => {
                if self.components.is_empty() {
                    return Err(CryptoError::InvalidAggregate(
                        "Type-2 requires at least one component".into(),
                    ));
                }
                if self.components.len() > crate::aggregation::bindings::MAX_TYPE2_COMPONENTS {
                    return Err(CryptoError::InvalidAggregate(
                        "Type-2 component count exceeds limit".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Deterministic digest of the statement (public-input binding).
    pub fn digest(&self) -> [u8; 32] {
        let mut buf = Vec::with_capacity(128 + self.participants.as_slice().len() * 4);
        buf.push(match self.kind {
            ProofKind::Type1 => 1,
            ProofKind::Type2 => 2,
        });
        buf.extend_from_slice(&self.profile_digest);
        buf.extend_from_slice(&self.message_root);
        buf.extend_from_slice(&self.slot.to_le_bytes());
        buf.extend_from_slice(&(self.participants.as_slice().len() as u32).to_le_bytes());
        for idx in self.participants.as_slice() {
            buf.extend_from_slice(&idx.to_le_bytes());
        }
        buf.extend_from_slice(&(self.components.len() as u32).to_le_bytes());
        for c in &self.components {
            buf.extend_from_slice(&c.message_root);
            buf.extend_from_slice(&c.slot.to_le_bytes());
        }
        domain_digest(b"ethean-crypto/v1/aggregate-statement", &buf)
    }
}

/// Reject oversized proof payloads early.
pub fn check_proof_len(proof: &[u8]) -> Result<()> {
    if proof.is_empty() {
        return Err(CryptoError::InvalidAggregate("empty proof".into()));
    }
    if proof.len() > MAX_PROOF_BYTES {
        return Err(CryptoError::InvalidAggregate(format!(
            "proof length {} exceeds {}",
            proof.len(),
            MAX_PROOF_BYTES
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_participants() {
        assert!(ParticipantSet::try_from_ordered(vec![1, 1]).is_err());
    }

    #[test]
    fn type1_requires_participants() {
        let s = AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [0u8; 32],
            message_root: [1u8; 32],
            slot: 3,
            participants: ParticipantSet::empty(),
            components: vec![],
        };
        assert!(s.validate_shape().is_err());
    }
}
