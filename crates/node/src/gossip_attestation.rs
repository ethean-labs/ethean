//! Attestation and aggregation gossip, following leanSpec
//! `on_gossip_attestation` and `on_gossip_aggregated_attestation`.
//!
//! Keys come from the local head state's registry. Structural checks mirror
//! the parts of `validate_attestation` that do not need a fork-choice store.

use ethean_crypto::{
    domain_digest, AggregateVerifier, CryptoBackend, ProductionBackend, Signature,
    PROD_AGGREGATION_FINGERPRINT,
};
use ethean_multisig::LeanMultisigVerifier;
use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, AggregationBits, AttestationData, SignedAggregatedAttestation,
    SignedAttestation,
};

use crate::aggregation::{PoolEntry, PoolKey};
use crate::chain_owner::ChainOwner;
use crate::registry_keys_view::{attestation_key, attestation_keys_for_bits};

/// Profile digest keying the aggregate pool.
pub fn pool_profile_digest() -> Hash32 {
    domain_digest(
        b"ethean-transition/v1/agg-profile",
        PROD_AGGREGATION_FINGERPRINT.as_bytes(),
    )
}

/// Pool key for an attestation data root.
pub fn pool_key(data_root: Hash32) -> PoolKey {
    PoolKey {
        profile_digest: pool_profile_digest(),
        message_root: data_root,
    }
}

/// Checkpoint ordering and a one-slot future bound on the vote slot.
fn check_data(owner: &ChainOwner, data: &AttestationData) -> Result<(), String> {
    if data.source.slot > data.target.slot || data.target.slot > data.head.slot {
        return Err("checkpoints out of order".into());
    }
    if let Some(tick) = owner.last_tick {
        if data.slot.get() > tick.slot.get() + 1 {
            return Err(format!("vote slot {} is in the future", data.slot.get()));
        }
    }
    Ok(())
}

/// Verify a gossiped vote; aggregators keep its signature for aggregation.
pub fn on_signed_attestation(owner: &mut ChainOwner, payload: &[u8]) -> Result<Hash32, String> {
    let vote = SignedAttestation::ssz_decode(payload).map_err(|e| e.to_string())?;
    check_data(owner, &vote.data)?;
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let key = attestation_key(state, vote.validator_index.get())?;
    let signature = Signature::try_from_slice(&vote.signature).map_err(|e| e.to_string())?;
    let data_root = vote.data.hash_tree_root();
    let slot = u32::try_from(vote.data.slot.get()).map_err(|_| "slot beyond XMSS lifetime")?;
    if !ProductionBackend
        .verify(&key, slot, &data_root, &signature)
        .map_err(|e| e.to_string())?
    {
        return Err("invalid attestation signature".into());
    }
    if owner.is_aggregator {
        owner
            .signatures
            .insert(data_root, &vote.data, vote.validator_index.get(), signature);
    }
    Ok(data_root)
}

/// Verify a gossiped aggregate against its participants and pool it.
pub fn on_signed_aggregate(owner: &mut ChainOwner, payload: &[u8]) -> Result<Hash32, String> {
    let aggregate = SignedAggregatedAttestation::ssz_decode(payload).map_err(|e| e.to_string())?;
    check_data(owner, &aggregate.data)?;
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let bits = &aggregate.proof.participants.bits;
    let keys = attestation_keys_for_bits(state, bits)?;
    let data_root = aggregate.data.hash_tree_root();
    LeanMultisigVerifier
        .verify_single(
            &aggregate.proof.proof,
            &keys,
            &data_root,
            aggregate.data.slot.get(),
        )
        .map_err(|e| e.to_string())?;
    insert_proved(owner, &aggregate.data, bits.clone(), aggregate.proof.proof)?;
    Ok(data_root)
}

/// Pool a verified Type-1 proof covering `participants`.
pub fn insert_proved(
    owner: &mut ChainOwner,
    data: &AttestationData,
    participants: Vec<bool>,
    proof: Vec<u8>,
) -> Result<(), String> {
    let coverage = participants.iter().filter(|b| **b).count() as u32;
    let attestation = AggregatedAttestation {
        aggregation_bits: AggregationBits::new(participants).map_err(|e| e.to_string())?,
        data: *data,
    };
    owner.aggregates.insert_verified(
        pool_key(data.hash_tree_root()),
        PoolEntry {
            proof,
            coverage,
            inserted_slot: data.slot.get(),
            attestation_ssz: attestation.ssz_encode(),
        },
    );
    Ok(())
}

/// Route attestation-subnet and aggregation-topic payloads. Returns the
/// attestation data root on acceptance.
pub fn ingest_attestation_gossip(
    owner: &mut ChainOwner,
    topic: &str,
    payload: &[u8],
) -> Option<Result<Hash32, String>> {
    if topic.contains("/attestation_") {
        return Some(on_signed_attestation(owner, payload));
    }
    if topic.contains("/aggregation/") {
        return Some(on_signed_aggregate(owner, payload));
    }
    None
}
