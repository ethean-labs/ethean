//! Attestation and aggregation gossip, following leanSpec
//! `on_gossip_attestation` and `on_gossip_aggregated_attestation`.
//!
//! Keys come from the local head state's registry. After signature / proof
//! checks succeed, votes are also offered to the live fork-choice store
//! (structural pending pool).

use std::time::{Duration, Instant};

use ethean_crypto::{
    domain_digest, verify_batch, AggregateVerifier, BatchVerifyItem, CryptoBackend,
    ProductionBackend, PublicKey, Signature, XmssPublicKey, PROD_AGGREGATION_FINGERPRINT,
};
use ethean_multisig::LeanMultisigVerifier;
use ethean_primitives::Hash32;
use ethean_types::{
    AggregatedAttestation, AggregationBits, AttestationData, SignedAggregatedAttestation,
    SignedAttestation,
};

use crate::aggregation::{PoolEntry, PoolKey};
use crate::chain_owner::ChainOwner;
use crate::lean_metrics;
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

/// Smallest number of votes worth fanning out across cores.
const MIN_VOTE_BATCH: usize = 2;

fn is_vote_topic(topic: &str) -> bool {
    topic.contains("/attestation_")
}

/// Signature verdict for one gossiped vote, computed ahead of admission so
/// the votes of a pump window verify in parallel.
#[derive(Debug, Clone)]
pub struct PreVerifiedVote {
    key: PublicKey,
    valid: bool,
    verification: Duration,
}

/// Everything signature verification needs from a decoded vote.
struct VoteInput {
    vote: SignedAttestation,
    key: PublicKey,
    signature: Signature,
    data_root: Hash32,
    slot: u32,
}

fn vote_input(owner: &ChainOwner, vote: SignedAttestation) -> Result<VoteInput, String> {
    check_data(owner, &vote.data)?;
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let key = attestation_key(state, vote.validator_index.get())?;
    let signature = Signature::try_from_slice(&vote.signature).map_err(|e| e.to_string())?;
    let data_root = vote.data.hash_tree_root();
    let slot = u32::try_from(vote.data.slot.get()).map_err(|_| "slot beyond XMSS lifetime")?;
    Ok(VoteInput {
        vote,
        key,
        signature,
        data_root,
        slot,
    })
}

/// Verify the signatures of every vote in `batch` (topic, payload) on all
/// cores. Index-aligned with `batch`; `None` for non-votes, for payloads the
/// cheap checks already reject (admission reports those), and for batches
/// too small to fan out.
pub fn preverify_votes(
    owner: &ChainOwner,
    batch: &[(&str, &[u8])],
) -> Vec<Option<PreVerifiedVote>> {
    let mut out = vec![None; batch.len()];
    let inputs: Vec<(usize, VoteInput)> = batch
        .iter()
        .enumerate()
        .filter(|(_, (topic, _))| is_vote_topic(topic))
        .filter_map(|(i, (_, payload))| {
            let vote = SignedAttestation::ssz_decode(payload).ok()?;
            vote_input(owner, vote).ok().map(|input| (i, input))
        })
        .collect();
    if inputs.len() < MIN_VOTE_BATCH {
        return out;
    }
    let started = Instant::now();
    let decoded: Vec<Option<XmssPublicKey>> = inputs
        .iter()
        .map(|(_, input)| ProductionBackend::decode_public_key(&input.key).ok())
        .collect();
    let mut positions = Vec::with_capacity(inputs.len());
    let mut items = Vec::with_capacity(inputs.len());
    for ((i, input), public_key) in inputs.iter().zip(&decoded) {
        let Some(public_key) = public_key else {
            out[*i] = Some(PreVerifiedVote {
                key: input.key,
                valid: false,
                verification: Duration::ZERO,
            });
            continue;
        };
        positions.push((*i, input.key));
        items.push(BatchVerifyItem {
            public_key,
            epoch: input.slot,
            message: &input.data_root,
            signature: &input.signature,
        });
    }
    let verdicts = verify_batch(&items);
    let per_vote = started.elapsed() / (items.len().max(1) as u32);
    for ((i, key), valid) in positions.into_iter().zip(verdicts) {
        out[i] = Some(PreVerifiedVote {
            key,
            valid,
            verification: per_vote,
        });
    }
    out
}

/// Verify a gossiped vote; aggregators keep its signature for aggregation.
pub fn on_signed_attestation(owner: &mut ChainOwner, payload: &[u8]) -> Result<Hash32, String> {
    on_signed_attestation_with(owner, payload, None)
}

/// [`on_signed_attestation`] reusing a verdict from [`preverify_votes`]. The
/// verdict only counts when it was made against the key the registry holds
/// now; otherwise the signature is verified again here.
pub fn on_signed_attestation_with(
    owner: &mut ChainOwner,
    payload: &[u8],
    pre: Option<PreVerifiedVote>,
) -> Result<Hash32, String> {
    let started = Instant::now();
    let mut verification = None;
    let mut ahead = Duration::ZERO;
    let result = check_signed_attestation(owner, payload, &mut verification, &mut ahead, pre);
    lean_metrics::attestation_checked(
        result.is_ok(),
        verification.is_some(),
        started.elapsed() + ahead,
        verification.unwrap_or_default(),
    );
    result
}

fn check_signed_attestation(
    owner: &mut ChainOwner,
    payload: &[u8],
    verification: &mut Option<Duration>,
    ahead: &mut Duration,
    pre: Option<PreVerifiedVote>,
) -> Result<Hash32, String> {
    let vote = SignedAttestation::ssz_decode(payload).map_err(|e| e.to_string())?;
    lean_metrics::gossip_attestation(owner, vote.data.slot.get(), payload.len());
    let input = vote_input(owner, vote)?;
    let valid = match pre.filter(|p| p.key == input.key) {
        Some(pre) => {
            *verification = Some(pre.verification);
            *ahead = pre.verification;
            pre.valid
        }
        None => {
            let verify_started = Instant::now();
            let valid = ProductionBackend.verify(
                &input.key,
                input.slot,
                &input.data_root,
                &input.signature,
            );
            *verification = Some(verify_started.elapsed());
            valid.map_err(|e| e.to_string())?
        }
    };
    if !valid {
        return Err("invalid attestation signature".into());
    }
    let VoteInput {
        vote,
        signature,
        data_root,
        ..
    } = input;
    if owner.is_aggregator && owner.aggregates_vote_of(vote.validator_index.get()) {
        owner
            .signatures
            .insert(data_root, &vote.data, vote.validator_index.get(), signature);
    }
    owner.fc_on_attestation(vote.validator_index, vote.data);
    Ok(data_root)
}

/// Verify a gossiped aggregate against its participants and pool it.
pub fn on_signed_aggregate(owner: &mut ChainOwner, payload: &[u8]) -> Result<Hash32, String> {
    let started = Instant::now();
    let mut verification = None;
    let result = check_signed_aggregate(owner, payload, &mut verification);
    lean_metrics::aggregate_checked(
        result.is_ok(),
        verification.is_some(),
        started.elapsed(),
        verification.unwrap_or_default(),
    );
    result
}

fn check_signed_aggregate(
    owner: &mut ChainOwner,
    payload: &[u8],
    verification: &mut Option<Duration>,
) -> Result<Hash32, String> {
    let aggregate = SignedAggregatedAttestation::ssz_decode(payload).map_err(|e| e.to_string())?;
    lean_metrics::gossip_aggregation(owner, payload.len());
    check_data(owner, &aggregate.data)?;
    let state = owner.head_state.as_ref().ok_or("no head state")?;
    let bits = aggregate.proof.participants.bits.clone();
    let keys = attestation_keys_for_bits(state, &bits)?;
    let data_root = aggregate.data.hash_tree_root();
    let verify_started = Instant::now();
    let checked = LeanMultisigVerifier.verify_single(
        &aggregate.proof.proof,
        &keys,
        &data_root,
        aggregate.data.slot.get(),
    );
    *verification = Some(verify_started.elapsed());
    checked.map_err(|e| e.to_string())?;
    insert_proved(owner, &aggregate.data, bits.clone(), aggregate.proof.proof)?;
    owner.fc_on_aggregated(aggregate.data, &bits);
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
    ingest_attestation_gossip_with(owner, topic, payload, None)
}

/// [`ingest_attestation_gossip`] with a verdict from [`preverify_votes`].
pub fn ingest_attestation_gossip_with(
    owner: &mut ChainOwner,
    topic: &str,
    payload: &[u8],
    pre: Option<PreVerifiedVote>,
) -> Option<Result<Hash32, String>> {
    if is_vote_topic(topic) {
        return Some(on_signed_attestation_with(owner, payload, pre));
    }
    if topic.contains("/aggregation/") {
        return Some(on_signed_aggregate(owner, payload));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethean_crypto::SIGNATURE_BYTES;
    use ethean_genesis::GenesisBuilder;
    use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
    use ethean_types::Checkpoint;

    const VOTE_TOPIC: &str = "/leanconsensus/abcd/attestation_0/ssz_snappy";

    fn aggregator(validators: usize) -> ChainOwner {
        let keys = vec![(Bytes52::ZERO, Bytes52::ZERO); validators];
        let genesis = GenesisBuilder::new(1_700_000_000)
            .with_validator_keys(keys)
            .build()
            .unwrap();
        let mut owner = ChainOwner::new(2);
        owner.head_state = Some(genesis.state);
        owner.is_aggregator = true;
        owner
    }

    fn vote(index: u64) -> Vec<u8> {
        let data = AttestationData {
            slot: Slot::new(0),
            head: Checkpoint::genesis(),
            target: Checkpoint::genesis(),
            source: Checkpoint::genesis(),
        };
        SignedAttestation::new(ValidatorIndex::new(index), data, vec![0u8; SIGNATURE_BYTES])
            .unwrap()
            .ssz_encode()
    }

    #[test]
    fn a_window_of_votes_is_verified_up_front_and_index_aligned() {
        let owner = aggregator(3);
        let (a, b, c) = (vote(0), vote(1), vote(9));
        let batch: Vec<(&str, &[u8])> = vec![
            (VOTE_TOPIC, &a),
            ("/leanconsensus/abcd/block/ssz_snappy", b"block"),
            (VOTE_TOPIC, &b),
            (VOTE_TOPIC, &c),
            (VOTE_TOPIC, b"garbage"),
        ];
        let verdicts = preverify_votes(&owner, &batch);
        assert_eq!(verdicts.len(), batch.len());
        assert!(!verdicts[0].as_ref().expect("vote verified").valid);
        assert!(verdicts[1].is_none(), "blocks are not votes");
        assert!(!verdicts[2].as_ref().expect("vote verified").valid);
        assert!(
            verdicts[3].is_none(),
            "unknown validator fails before verify"
        );
        assert!(
            verdicts[4].is_none(),
            "undecodable payload fails before verify"
        );
    }

    #[test]
    fn a_lone_vote_is_left_to_inline_verification() {
        let owner = aggregator(1);
        let a = vote(0);
        assert!(preverify_votes(&owner, &[(VOTE_TOPIC, &a)])[0].is_none());
    }

    #[test]
    fn a_verdict_only_counts_for_the_key_it_was_made_against() {
        let mut owner = aggregator(2);
        let stale = PreVerifiedVote {
            key: PublicKey::from_bytes([1u8; 52]),
            valid: true,
            verification: Duration::ZERO,
        };
        assert!(on_signed_attestation_with(&mut owner, &vote(0), Some(stale)).is_err());
        assert!(owner.signatures.is_empty());

        let current = PreVerifiedVote {
            key: PublicKey::from_bytes([0u8; 52]),
            valid: true,
            verification: Duration::ZERO,
        };
        assert!(on_signed_attestation_with(&mut owner, &vote(1), Some(current)).is_ok());
        assert!(!owner.signatures.is_empty());
    }
}
