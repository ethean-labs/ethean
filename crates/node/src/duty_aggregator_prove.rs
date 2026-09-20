//! Type-1 prove dispatch when aggregator coverage clears.
//!
//! Uses [`ProverWorker`] → `prove_type1`, which prefers leanVM process IPC when
//! `ETHEAN_LEANVM_PROVER` is set (else test-aggregate smoke proofs).

use crate::aggregation::{AggregationBudget, PoolEntry, PoolKey, ProverJob, ProverOutcome, ProverWorker};
use crate::chain_owner::ChainOwner;
use crate::events::ChainEvent;
use crate::gossip_pool::pool_profile_digest;
use ethean_crypto::{verify_type1, AggregateStatement, ParticipantSet, ProofKind};
use ethean_primitives::Hash32;
use ethean_types::AggregatedAttestation;

/// Attempt a Type-1 prove for one ready attestation-data root and refresh the pool.
pub fn try_prove_type1_for_root(owner: &mut ChainOwner, data_root: Hash32) -> Option<ChainEvent> {
    let profile = pool_profile_digest();
    let key = PoolKey {
        profile_digest: profile,
        message_root: data_root,
    };
    let entry = owner.aggregates.best(&key)?.clone();
    if entry.attestation_ssz.is_empty() {
        return None;
    }
    let att = AggregatedAttestation::ssz_decode(&entry.attestation_ssz).ok()?;
    let statement = statement_from_attestation(profile, &att).ok()?;
    let worker = ProverWorker::new(AggregationBudget::default());
    match worker.run(ProverJob {
        statement: statement.clone(),
    }) {
        ProverOutcome::Produced(proof) => match verify_type1(&statement, &proof) {
            Ok(()) => {
                let proof_len = proof.len();
                owner.aggregates.insert_verified(
                    key,
                    PoolEntry {
                        proof,
                        coverage: entry.coverage.max(1),
                        inserted_slot: entry.inserted_slot,
                        attestation_ssz: entry.attestation_ssz,
                    },
                );
                Some(ChainEvent::AggregatorType1Proved {
                    data_root,
                    proof_len,
                })
            }
            Err(_) => None,
        },
        ProverOutcome::Failed(_) | ProverOutcome::Budget(_) => None,
    }
}

fn statement_from_attestation(
    profile: [u8; 32],
    att: &AggregatedAttestation,
) -> Result<AggregateStatement, String> {
    let mut indices = Vec::new();
    for (i, bit) in att.aggregation_bits.bits.iter().enumerate() {
        if *bit {
            indices.push(i as u32);
        }
    }
    let participants = ParticipantSet::try_from_ordered(indices).map_err(|e| e.to_string())?;
    Ok(AggregateStatement {
        kind: ProofKind::Type1,
        profile_digest: profile,
        message_root: att.data.hash_tree_root(),
        slot: att.data.slot.get(),
        participants,
        components: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain_owner::ChainOwner;
    use ethean_primitives::Slot;
    use ethean_types::{AggregatedAttestation, AggregationBits, AttestationData, Checkpoint};

    #[test]
    fn proves_when_attestation_ssz_present() {
        let mut owner = ChainOwner::default();
        let att = AggregatedAttestation {
            aggregation_bits: AggregationBits {
                bits: vec![true, true],
            },
            data: AttestationData {
                slot: Slot::new(1),
                head: Checkpoint::genesis(),
                target: Checkpoint::genesis(),
                source: Checkpoint::genesis(),
            },
        };
        let data_root = att.data.hash_tree_root();
        let key = PoolKey {
            profile_digest: pool_profile_digest(),
            message_root: data_root,
        };
        owner.aggregates.insert_verified(
            key,
            PoolEntry {
                proof: Vec::new(),
                coverage: 2,
                inserted_slot: 1,
                attestation_ssz: att.ssz_encode(),
            },
        );
        let ev = try_prove_type1_for_root(&mut owner, data_root);
        // test-aggregate builds attach; production refuse stays None.
        match ev {
            Some(ChainEvent::AggregatorType1Proved { proof_len, .. }) => {
                assert!(proof_len > 0);
                assert!(!owner.aggregates.best(&key).unwrap().proof.is_empty());
            }
            None => assert!(owner.aggregates.best(&key).unwrap().proof.is_empty()),
            other => panic!("unexpected {other:?}"),
        }
    }
}
