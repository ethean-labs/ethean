//! End to end over real cryptography: native XMSS votes -> gossip -> Type-1
//! aggregate (prover process) -> aggregation gossip -> block with a merged
//! Type-2 proof -> verified import on a second node.
//!
//! Needs `leansig-test-keys/prod_scheme` next to the workspace and a built
//! `ethean-prover` (or `ETHEAN_PROVER_BIN`); skips otherwise. Run with
//! `cargo build --release -p ethean-prover && cargo test --release -p ethean-node --test aggregation_flow`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ethean_crypto::{CryptoBackend, ProductionBackend, PublicKey, SecretKeyMaterial};
use ethean_genesis::GenesisBuilder;
use ethean_multisig::ProverConfig;
use ethean_node::chain_owner::ChainOwner;
use ethean_node::events::ChainEvent;
use ethean_node::gossip_attestation::ingest_attestation_gossip;
use ethean_node::local_proposer::LocalProposer;
use ethean_node::proof_service::ProofService;
use ethean_node::shutdown::ShutdownState;
use ethean_node::{aggregation_duty, duty_propose, gossip_decode, gossip_stf, proof_collect};
use ethean_primitives::{Bytes52, Slot, ValidatorIndex};
use ethean_types::{AttestationData, Checkpoint, SignedAttestation};
use ethean_validator::{DutyTick, KeyId, KeyRecord, SigningRole};

const VALIDATORS: usize = 4;

struct Keys {
    attestation: (PublicKey, SecretKeyMaterial),
    proposal: (PublicKey, SecretKeyMaterial),
}

fn hex_field(doc: &str, key: &str) -> Vec<u8> {
    let needle = format!("\"{key}\": \"");
    let start = doc.find(&needle).unwrap() + needle.len();
    let end = doc[start..].find('"').unwrap() + start;
    let hex = &doc[start..end];
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn keypair(doc: &str, role: &str) -> (PublicKey, SecretKeyMaterial) {
    let section = &doc[doc.find(role).unwrap()..];
    (
        PublicKey::try_from_slice(&hex_field(section, "public_key")).unwrap(),
        SecretKeyMaterial::from_xmss_ssz(hex_field(section, "secret_key")).unwrap(),
    )
}

fn load_keys() -> Option<Vec<Keys>> {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../leansig-test-keys/prod_scheme");
    (0..VALIDATORS)
        .map(|i| {
            let doc = std::fs::read_to_string(dir.join(format!("{i}.json"))).ok()?;
            Some(Keys {
                attestation: keypair(&doc, "attestation_keypair"),
                proposal: keypair(&doc, "proposal_keypair"),
            })
        })
        .collect()
}

fn prover_config() -> Option<ProverConfig> {
    if let Some(config) = ProverConfig::discover() {
        return Some(config);
    }
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let bin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/{profile}/ethean-prover"));
    bin.is_file().then(|| ProverConfig::new(bin))
}

fn owner(keys: &[Keys]) -> ChainOwner {
    let registry = keys
        .iter()
        .map(|k| {
            (
                Bytes52(*k.attestation.0.as_bytes()),
                Bytes52(*k.proposal.0.as_bytes()),
            )
        })
        .collect();
    let genesis = GenesisBuilder::new(1_700_000_000)
        .with_validator_keys(registry)
        .build()
        .unwrap();
    let mut owner = ChainOwner::new(8);
    owner.head_state = Some(genesis.state);
    owner.profile = Some(ethean_profile::lstar_devnet().unwrap());
    ethean_node::local_finality::seal_genesis_head(&mut owner);
    owner
}

fn wait_for(owner: &mut ChainOwner, want: impl Fn(&ChainEvent) -> bool) -> Vec<ChainEvent> {
    let deadline = Instant::now() + Duration::from_secs(900);
    let mut seen = Vec::new();
    while Instant::now() < deadline {
        let events = proof_collect::collect_proofs(owner);
        let done = events.iter().any(&want);
        seen.extend(events);
        if done {
            return seen;
        }
        if let Some(ChainEvent::ProofFailed { kind, error }) = seen.last() {
            panic!("{kind} proof failed: {error}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("timed out waiting for proof; saw {seen:?}");
}

#[test]
fn votes_aggregate_into_a_block_that_a_peer_verifies() {
    let (Some(keys), Some(prover)) = (load_keys(), prover_config()) else {
        eprintln!("skipping: needs leansig-test-keys/prod_scheme and a built ethean-prover");
        return;
    };
    let mut aggregator = owner(&keys);
    aggregator.is_aggregator = true;
    aggregator.prover = Some(ProofService::spawn(prover).unwrap());
    let mut peer = owner(&keys);
    assert_eq!(aggregator.head_root, peer.head_root);

    // Validators 0, 2 and 3 vote for the genesis checkpoint at slot 0.
    let genesis = Checkpoint {
        root: aggregator.head_root,
        slot: Slot::new(0),
    };
    let data = AttestationData {
        slot: Slot::new(0),
        head: genesis,
        target: genesis,
        source: genesis,
    };
    let data_root = data.hash_tree_root();
    for index in [0usize, 2, 3] {
        let signature = ProductionBackend
            .sign(&keys[index].attestation.1, 0, &data_root)
            .unwrap();
        let vote = SignedAttestation::new(
            ValidatorIndex::new(index as u64),
            data,
            signature.as_bytes().to_vec(),
        )
        .unwrap();
        let accepted = ingest_attestation_gossip(
            &mut aggregator,
            "/leanconsensus/x/attestation_0/ssz_snappy",
            &vote.ssz_encode(),
        );
        assert_eq!(accepted, Some(Ok(data_root)));
    }
    let forged = SignedAttestation::new(
        ValidatorIndex::new(1),
        data,
        ProductionBackend
            .sign(&keys[0].attestation.1, 1, &data_root)
            .unwrap()
            .as_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(matches!(
        ingest_attestation_gossip(&mut aggregator, "/x/attestation_0/y", &forged.ssz_encode()),
        Some(Err(_))
    ));

    // Aggregate, then deliver the published aggregate to the peer.
    let scheduled = aggregation_duty::schedule_aggregations(&mut aggregator);
    assert!(matches!(
        scheduled.as_slice(),
        [ChainEvent::ProofScheduled {
            kind: "attestation",
            ..
        }]
    ));
    let events = wait_for(&mut aggregator, |e| {
        matches!(e, ChainEvent::AggregateProved { .. })
    });
    assert!(events
        .iter()
        .any(|e| matches!(e, ChainEvent::AggregateProved { coverage: 3, .. })));
    let aggregate = aggregator
        .pending_aggregation_gossip
        .pop()
        .expect("aggregate queued");
    assert!(aggregate.topic.contains("/aggregation/"));
    assert_eq!(
        ingest_attestation_gossip(&mut peer, &aggregate.topic, &aggregate.payload),
        Some(Ok(data_root))
    );
    assert_eq!(peer.aggregates.len(), 1);

    // Validator 1 proposes slot 1 with the aggregate in its body.
    let (proposal_key, proposal_secret) = keys[1].proposal.clone();
    let record = KeyRecord {
        key_id: KeyId::from_bytes([0xb2; 16]),
        role: SigningRole::Proposal,
        public_key: proposal_key,
        secret: proposal_secret,
        activation_slot: 0,
        num_active_slots: 131_072,
        journal_generation: 1,
    };
    aggregator.proposer = Some(LocalProposer::from_key_record(record).unwrap());
    let tick = DutyTick {
        slot: Slot::new(1),
        interval: 0,
        generation: 1,
    };
    let planned = duty_propose::try_plan_proposal(&mut aggregator, tick);
    assert!(
        planned.iter().any(|e| matches!(
            e,
            ChainEvent::ProposalPlanned {
                attestations: 1,
                ..
            }
        )),
        "{planned:?}"
    );
    assert!(
        planned
            .iter()
            .any(|e| matches!(e, ChainEvent::ProofScheduled { kind: "block", .. })),
        "{planned:?}"
    );
    wait_for(&mut aggregator, |e| {
        matches!(e, ChainEvent::BlockProofAttached { .. })
    });
    let block = aggregator
        .pending_block_gossip
        .clone()
        .expect("block queued");
    assert_eq!(aggregator.head_root, block.block_root);

    // A tampered proof is rejected; the real block imports after verification.
    let decoded = gossip_decode::try_decode_block(&block.topic, &block.payload).unwrap();
    let mut tampered = decoded.clone();
    let mid = tampered.signed.proof.proof.len() / 2;
    tampered.signed.proof.proof[mid] ^= 1;
    let shutdown = ShutdownState::default();
    assert!(matches!(
        gossip_stf::import_decoded_block(&mut peer, &shutdown, &tampered),
        gossip_stf::GossipStfResult::Rejected { .. }
    ));
    assert_eq!(
        gossip_stf::import_decoded_block(&mut peer, &shutdown, &decoded),
        gossip_stf::GossipStfResult::Applied {
            root: block.block_root
        }
    );
    assert_eq!(peer.head_root, aggregator.head_root);

    // leanMetrics: the same process recorded every step above.
    use ethean_metrics::lean::{observations, value};
    let count = |name| value(name, &[]).unwrap_or(0.0);
    assert!(count("lean_pq_sig_attestation_signatures_valid_total") >= 3.0);
    assert!(count("lean_pq_sig_attestation_signatures_invalid_total") >= 1.0);
    assert!(count("lean_pq_sig_aggregated_signatures_total") >= 1.0);
    assert!(count("lean_pq_sig_attestations_in_aggregated_signatures_total") >= 3.0);
    assert!(count("lean_pq_sig_aggregated_signatures_valid_total") >= 1.0);
    assert!(count("lean_block_building_success_total") >= 1.0);
    assert!(count("lean_state_transition_attestations_processed_total") >= 1.0);
    for histogram in [
        "lean_pq_sig_attestation_verification_time_seconds",
        "lean_pq_sig_aggregated_signatures_building_time_seconds",
        "lean_pq_sig_aggregated_signatures_verification_time_seconds",
        "lean_block_aggregated_payloads",
        "lean_block_building_payload_aggregation_time_seconds",
        "lean_fork_choice_block_processing_time_seconds",
        "lean_state_transition_time_seconds",
        "lean_gossip_attestation_size_bytes",
        "lean_gossip_aggregation_size_bytes",
    ] {
        assert!(observations(histogram, &[]).unwrap_or(0) >= 1, "{histogram} not observed");
    }
}
