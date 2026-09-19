//! Type-1 statement helpers from SingleMessageAggregate.

use ethean_primitives::Hash32;

use crate::aggregate::SingleMessageAggregate;
use crate::error::TypesError;
use crate::proofs::participants::indices_from_bits;

/// Build ordered participant indices and message binding for a Type-1 aggregate.
///
/// Returns `(message_root, slot, ordered_indices)`. The caller supplies the
/// consensus-derived message root and slot — wire metadata must not override them.
pub fn type1_statement_from_aggregate(
    aggregate: &SingleMessageAggregate,
    message_root: Hash32,
    slot: u64,
) -> Result<(Hash32, u64, Vec<u32>), TypesError> {
    if aggregate.proof.is_empty() {
        return Err(TypesError::InvalidContainer(
            "Type-1 proof bytes are empty".into(),
        ));
    }
    let indices = indices_from_bits(&aggregate.participants)?;
    if indices.is_empty() {
        return Err(TypesError::InvalidContainer(
            "Type-1 requires at least one participant".into(),
        ));
    }
    Ok((message_root, slot, indices))
}
