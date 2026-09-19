//! Participant index helpers for aggregate bitfields.

use crate::aggregate::AggregationBits;
use crate::error::TypesError;
use crate::limits::VALIDATOR_REGISTRY_LIMIT;

/// Validate strictly increasing unique indices within the registry.
pub fn validate_ordered_indices(indices: &[u32]) -> Result<(), TypesError> {
    if indices.len() > VALIDATOR_REGISTRY_LIMIT {
        return Err(TypesError::ListTooLong {
            got: indices.len(),
            limit: VALIDATOR_REGISTRY_LIMIT,
        });
    }
    for (i, &idx) in indices.iter().enumerate() {
        if idx as usize >= VALIDATOR_REGISTRY_LIMIT {
            return Err(TypesError::ValidatorIndexOutOfRange {
                index: idx as u64,
                limit: VALIDATOR_REGISTRY_LIMIT as u64,
            });
        }
        if i > 0 && indices[i - 1] >= idx {
            return Err(TypesError::InvalidContainer(
                "participant indices must be strictly increasing".into(),
            ));
        }
    }
    Ok(())
}

/// Collect set bits as ordered validator indices.
pub fn indices_from_bits(bits: &AggregationBits) -> Result<Vec<u32>, TypesError> {
    let mut out = Vec::new();
    for (i, bit) in bits.bits.iter().enumerate() {
        if *bit {
            out.push(i as u32);
        }
    }
    validate_ordered_indices(&out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsorted() {
        assert!(validate_ordered_indices(&[2, 1]).is_err());
    }

    #[test]
    fn bits_to_indices() {
        let bits = AggregationBits::new(vec![true, false, true]).unwrap();
        assert_eq!(indices_from_bits(&bits).unwrap(), vec![0, 2]);
    }
}
