//! Aggregated attestation payload pool helpers (leanSpec new/known payloads).

use std::collections::HashMap;

use ethean_primitives::Hash32;
use ethean_types::AttestationData;

use super::state::{AggregatedPayloadEntry, ForkChoiceStore};

fn indices_from_bits(bits: &[bool]) -> Vec<u64> {
    bits.iter()
        .enumerate()
        .filter_map(|(i, b)| b.then_some(i as u64))
        .collect()
}

fn upsert_payload(
    map: &mut HashMap<Hash32, AggregatedPayloadEntry>,
    data: AttestationData,
    participants: Vec<u64>,
) {
    if participants.is_empty() {
        return;
    }
    let data_root = data.hash_tree_root();
    let entry = map.entry(data_root).or_insert_with(|| AggregatedPayloadEntry {
        data_root,
        data,
        participant_sets: Vec::new(),
    });
    if !entry.participant_sets.iter().any(|s| s == &participants) {
        entry.participant_sets.push(participants);
    }
}

impl ForkChoiceStore {
    /// Record an on-chain aggregate into the known payload pool.
    pub fn record_known_payload(&mut self, data: AttestationData, bits: &[bool]) {
        upsert_payload(
            &mut self.latest_known_payloads,
            data,
            indices_from_bits(bits),
        );
    }

    /// Record a gossip aggregate into the pending (new) payload pool.
    pub fn record_new_payload(&mut self, data: AttestationData, bits: &[bool]) {
        upsert_payload(
            &mut self.latest_new_payloads,
            data,
            indices_from_bits(bits),
        );
    }

    /// Move pending payloads into the known pool (merge participant sets).
    pub fn promote_new_payloads(&mut self) {
        let pending: Vec<_> = self.latest_new_payloads.drain().map(|(_, e)| e).collect();
        for entry in pending {
            for set in entry.participant_sets {
                upsert_payload(&mut self.latest_known_payloads, entry.data, set);
            }
        }
    }

    /// Drop known payloads whose data root is no longer referenced by a known vote.
    pub fn prune_unreferenced_payloads(&mut self) {
        let live: std::collections::HashSet<Hash32> = self
            .latest_known_attestations
            .values()
            .map(|d| d.hash_tree_root())
            .collect();
        self.latest_known_payloads
            .retain(|root, _| live.contains(root));
    }
}

/// Sort participant indices for stable fixture comparison.
pub fn normalized_participant_sets(sets: &[Vec<u64>]) -> Vec<Vec<u64>> {
    let mut out: Vec<Vec<u64>> = sets
        .iter()
        .map(|s| {
            let mut v = s.clone();
            v.sort_unstable();
            v
        })
        .collect();
    out.sort();
    out
}
