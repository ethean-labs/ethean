//! Startup reconciliation — never rewind counters; burn uncertain leaves.

use crate::error::Result;
use crate::signer::duty::SigningDuty;
use crate::signer::journal::SignerStore;

/// On restart, burn any non-durable pending reservations for `duties`.
/// Completed reservations are left intact. Counters never move backward.
pub fn reconcile_on_startup<S: SignerStore>(
    store: &mut S,
    uncertain: &[SigningDuty],
) -> Result<usize> {
    let mut burned = 0usize;
    for duty in uncertain {
        if store.completed_signature(duty)?.is_some() {
            continue;
        }
        if store.is_reserved_durable(duty)? {
            // Durable but incomplete: burn the leaf (conservative).
            store.burn_uncertain(duty)?;
            burned += 1;
            continue;
        }
        // Non-durable pending: drop without publishing a signature.
        store.burn_uncertain(duty)?;
        burned += 1;
    }
    Ok(burned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::duty::{KeyId, SigningRole, SigningRoot};
    use crate::signer::journal::InMemorySignerStore;

    #[test]
    fn burns_uncertain_pending() {
        let mut store = InMemorySignerStore::default();
        let duty = SigningDuty {
            key_id: KeyId::from_bytes([2u8; 16]),
            role: SigningRole::Attestation,
            slot: 9,
            root: SigningRoot::from_bytes([3u8; 32]),
        };
        let _ = store.reserve(&duty).unwrap();
        let n = reconcile_on_startup(&mut store, &[duty.clone()]).unwrap();
        assert_eq!(n, 1);
        assert!(store.is_burned(&duty).unwrap());
    }
}
