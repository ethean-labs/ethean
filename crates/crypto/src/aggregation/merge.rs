//! Type-1 recursive merge: union participants, then re-prove (fail-closed without leanVM).

use crate::aggregation::prove::prove_type1;
use crate::aggregation::statement::{AggregateStatement, ParticipantSet, ProofKind};
use crate::aggregation::verify::verify_type1;
use crate::error::{CryptoError, Result};

/// Merge two Type-1 statements over the same `(profile, message, slot)`.
///
/// Participant sets are unioned into a strictly increasing list. Does not touch proofs.
pub fn merge_type1_statements(
    left: &AggregateStatement,
    right: &AggregateStatement,
) -> Result<AggregateStatement> {
    if left.kind != ProofKind::Type1 || right.kind != ProofKind::Type1 {
        return Err(CryptoError::InvalidAggregate(
            "merge_type1_statements requires two Type-1 statements".into(),
        ));
    }
    left.validate_shape()?;
    right.validate_shape()?;
    if left.profile_digest != right.profile_digest
        || left.message_root != right.message_root
        || left.slot != right.slot
    {
        return Err(CryptoError::InvalidAggregate(
            "Type-1 merge requires matching profile, message_root, and slot".into(),
        ));
    }
    let participants = union_participants(&left.participants, &right.participants)?;
    Ok(AggregateStatement {
        kind: ProofKind::Type1,
        profile_digest: left.profile_digest,
        message_root: left.message_root,
        slot: left.slot,
        participants,
        components: vec![],
    })
}

/// Verify two Type-1 proofs, merge their statements, and produce a merged proof.
///
/// Production path uses leanVM via [`prove_type1`] (fail-closed when unavailable).
/// With `test-aggregate`, the merged statement gets a fresh statement-bound test proof.
pub fn merge_type1(
    left: &AggregateStatement,
    left_proof: &[u8],
    right: &AggregateStatement,
    right_proof: &[u8],
) -> Result<(AggregateStatement, Vec<u8>)> {
    verify_type1(left, left_proof)?;
    verify_type1(right, right_proof)?;
    let merged = merge_type1_statements(left, right)?;
    // Identical participant sets: keep the left proof (idempotent merge).
    if merged.participants.as_slice() == left.participants.as_slice() {
        return Ok((merged, left_proof.to_vec()));
    }
    if merged.participants.as_slice() == right.participants.as_slice() {
        return Ok((merged, right_proof.to_vec()));
    }
    let proof = prove_type1(&merged)?;
    Ok((merged, proof))
}

fn union_participants(a: &ParticipantSet, b: &ParticipantSet) -> Result<ParticipantSet> {
    let mut merged = Vec::with_capacity(a.as_slice().len() + b.as_slice().len());
    let (mut i, mut j) = (0usize, 0usize);
    let left = a.as_slice();
    let right = b.as_slice();
    while i < left.len() && j < right.len() {
        match left[i].cmp(&right[j]) {
            std::cmp::Ordering::Less => {
                merged.push(left[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                merged.push(right[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                merged.push(left[i]);
                i += 1;
                j += 1;
            }
        }
    }
    merged.extend_from_slice(&left[i..]);
    merged.extend_from_slice(&right[j..]);
    ParticipantSet::try_from_ordered(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stmt(participants: Vec<u32>) -> AggregateStatement {
        AggregateStatement {
            kind: ProofKind::Type1,
            profile_digest: [1u8; 32],
            message_root: [2u8; 32],
            slot: 9,
            participants: ParticipantSet::try_from_ordered(participants).unwrap(),
            components: vec![],
        }
    }

    #[test]
    fn unions_participants() {
        let a = stmt(vec![0, 2]);
        let b = stmt(vec![1, 2, 4]);
        let m = merge_type1_statements(&a, &b).unwrap();
        assert_eq!(m.participants.as_slice(), &[0, 1, 2, 4]);
    }

    #[test]
    fn rejects_message_mismatch() {
        let mut b = stmt(vec![1]);
        b.message_root = [9u8; 32];
        assert!(merge_type1_statements(&stmt(vec![0]), &b).is_err());
    }

    #[test]
    fn merge_proofs_roundtrip() {
        let a = stmt(vec![0, 1]);
        let b = stmt(vec![1, 3]);
        let pa = prove_type1(&a).unwrap();
        let pb = prove_type1(&b).unwrap();
        let (merged, proof) = merge_type1(&a, &pa, &b, &pb).unwrap();
        assert_eq!(merged.participants.as_slice(), &[0, 1, 3]);
        verify_type1(&merged, &proof).unwrap();
    }
}
