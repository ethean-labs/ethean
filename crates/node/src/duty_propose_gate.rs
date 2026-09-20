//! Owned-index proposer assignment gate for Hive / registry nodes.

use crate::chain_owner::ChainOwner;

/// True when this node should produce the proposal for `slot`.
///
/// An empty `owned_validator_indices` list means solo/smoke mode: always act as
/// the protocol-assigned proposer (`slot % n`). With Hive registry indices set,
/// only propose when that assignment is owned locally.
pub fn is_assigned_proposer(owner: &ChainOwner, slot: u64, validator_count: u64) -> bool {
    if validator_count == 0 {
        return false;
    }
    if owner.owned_validator_indices.is_empty() {
        return true;
    }
    let assigned = slot % validator_count;
    owner.owned_validator_indices.contains(&assigned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_owned_allows_all() {
        let owner = ChainOwner::new(2);
        assert!(is_assigned_proposer(&owner, 1, 3));
        assert!(is_assigned_proposer(&owner, 0, 3));
    }

    #[test]
    fn respects_owned_indices() {
        let mut owner = ChainOwner::new(2);
        owner.owned_validator_indices = vec![0, 2];
        assert!(is_assigned_proposer(&owner, 0, 3));
        assert!(!is_assigned_proposer(&owner, 1, 3));
        assert!(is_assigned_proposer(&owner, 2, 3));
        assert!(is_assigned_proposer(&owner, 3, 3));
    }

    #[test]
    fn zero_validators_never_assigned() {
        let owner = ChainOwner::new(2);
        assert!(!is_assigned_proposer(&owner, 0, 0));
    }
}
