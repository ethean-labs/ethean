//! Prune stale attestation data after finalization advances (leanSpec).

use ethean_primitives::Hash32;

use crate::store::ForkChoiceStore;

/// Drop votes and payloads whose head can no longer influence fork choice.
///
/// leanSpec filters attestation-keyed pools only — it does **not** delete
/// blocks from the store when finalization advances. A vote/payload survives
/// when its head is strictly above the finalized slot and a descendant of the
/// finalized checkpoint.
pub fn prune_stale_attestation_data(store: &mut ForkChoiceStore) {
    let finalized = store.latest_finalized;

    let drop_new: Vec<_> = store
        .latest_new_attestations
        .iter()
        .filter(|(_, data)| {
            !(data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_new {
        store.latest_new_attestations.remove(&k);
    }

    let drop_known: Vec<_> = store
        .latest_known_attestations
        .iter()
        .filter(|(_, data)| {
            !(data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_known {
        store.latest_known_attestations.remove(&k);
    }

    let drop_new_payloads: Vec<Hash32> = store
        .latest_new_payloads
        .iter()
        .filter(|(_, entry)| {
            !(entry.data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, entry.data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_new_payloads {
        store.latest_new_payloads.remove(&k);
    }

    let drop_known_payloads: Vec<Hash32> = store
        .latest_known_payloads
        .iter()
        .filter(|(_, entry)| {
            !(entry.data.head.slot > finalized.slot
                && store.checkpoint_is_ancestor(finalized, entry.data.head))
        })
        .map(|(k, _)| *k)
        .collect();
    for k in drop_known_payloads {
        store.latest_known_payloads.remove(&k);
    }
}
