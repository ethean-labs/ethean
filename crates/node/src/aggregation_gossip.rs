//! Encode Type-1 aggregates for Lean `/aggregation/` and attestation subnet gossip.

use crate::aggregation::{PoolEntry, PoolKey};
use crate::chain_owner::ChainOwner;
use crate::gossip_pool::pool_profile_digest;
use ethean_network_wire::{fork_segment_from_name, topic_aggregation, topic_attestation};
use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, AggregationBits, SignedAggregatedAttestation, SingleMessageAggregate,
};

/// Gossip-ready Type-1 payload waiting for QuicSwarm publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregationGossip {
    /// Lean gossip topic string.
    pub topic: String,
    /// Uncompressed SSZ (or raw MultiMessageAggregate proof) bytes.
    pub payload: Vec<u8>,
    /// Attestation-data tree root.
    pub data_root: Hash32,
    /// Proof byte length carried in the payload.
    pub proof_len: usize,
}

/// Build `/aggregation/` + attestation-subnet gossip from a proved pool entry.
pub fn encode_type1_aggregation_gossip(
    entry: &PoolEntry,
    data_root: Hash32,
    subnet: u16,
    fork_name: &str,
) -> Result<Vec<AggregationGossip>, String> {
    if entry.proof.is_empty() || entry.attestation_ssz.is_empty() {
        return Err("type-1 gossip needs proof and attestation SSZ".into());
    }
    let att = AggregatedAttestation::ssz_decode(&entry.attestation_ssz).map_err(|e| e.to_string())?;
    let participants = AggregationBits::new(att.aggregation_bits.bits.clone()).map_err(|e| e.to_string())?;
    let single = SingleMessageAggregate::new(participants, entry.proof.clone())
        .map_err(|e| e.to_string())?;
    let signed = SignedAggregatedAttestation {
        data: att.data.clone(),
        proof: single,
    };
    let signed_payload = signed.ssz_encode().map_err(|e| e.to_string())?;

    let fork = fork_segment_from_name(fork_name).map_err(|e| e.to_string())?;
    let aggregation_topic = topic_aggregation(&fork).map_err(|e| e.to_string())?;
    let attestation_topic = topic_attestation(&fork, subnet).map_err(|e| e.to_string())?;

    Ok(vec![
        AggregationGossip {
            topic: aggregation_topic,
            payload: signed_payload.clone(),
            data_root,
            proof_len: entry.proof.len(),
        },
        AggregationGossip {
            topic: attestation_topic,
            payload: signed_payload,
            data_root,
            proof_len: entry.proof.len(),
        },
    ])
}

/// Queue Type-1 gossip for a proved root when the pool still holds attestation SSZ.
///
/// Returns the gossip items that were appended (empty when skipped).
pub fn queue_type1_aggregation_gossip(
    owner: &mut ChainOwner,
    data_root: Hash32,
    subnet: u16,
) -> Vec<AggregationGossip> {
    let Some(fork_name) = owner.profile.as_ref().map(|p| p.fork_name) else {
        return Vec::new();
    };
    let key = PoolKey {
        profile_digest: pool_profile_digest(),
        message_root: data_root,
    };
    let Some(entry) = owner.aggregates.best(&key).cloned() else {
        return Vec::new();
    };
    let Ok(items) = encode_type1_aggregation_gossip(&entry, data_root, subnet, fork_name) else {
        return Vec::new();
    };
    owner.pending_aggregation_gossip.extend(items.iter().cloned());
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_primitives::Slot;
    use ethean_types::{AttestationData, Checkpoint};

    #[test]
    fn encodes_aggregation_and_attestation_topics() {
        let att = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, false, true],
            },
            data: AttestationData {
                slot: Slot::new(4),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let entry = PoolEntry {
            proof: vec![7, 7, 7, 7],
            coverage: 2,
            inserted_slot: 4,
            attestation_ssz: att.ssz_encode(),
        };
        let root = att.data.hash_tree_root();
        let items = encode_type1_aggregation_gossip(&entry, root, 1, "lstar").expect("enc");
        assert_eq!(items.len(), 2);
        assert!(items[0].topic.ends_with("/aggregation/ssz_snappy"));
        assert!(items[1].topic.contains("/attestation_1/"));
        let decoded = SignedAggregatedAttestation::ssz_decode(&items[0].payload).expect("signed");
        assert_eq!(decoded.proof.proof, vec![7, 7, 7, 7]);
        assert_eq!(decoded.data.hash_tree_root(), root);
        assert_eq!(items[0].payload, items[1].payload);
    }
}
