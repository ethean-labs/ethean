//! Native XMSS signatures -> leanMultisig Type-1 -> Type-2 -> verification.

mod support;

use ethean_crypto::{ProofComponent, PublicKey};
use ethean_multisig::prove::{aggregate_type1, merge_type2, split_type2};
use ethean_multisig::{verify_multi, verify_single, KeyedProof, MultisigError};
use leansig_wrapper::{xmss_public_key_from_ssz, xmss_signature_from_ssz, xmss_verify};
use support::{prod_key, sign, ProdKey};

fn attesters(n: usize) -> Option<Vec<ProdKey>> {
    (0..n).map(|i| prod_key(i, "attestation_keypair")).collect()
}

#[test]
fn native_signatures_aggregate_merge_and_verify() {
    let Some(voters) = attesters(3) else {
        eprintln!("skipping: leansig-test-keys/prod_scheme not found");
        return;
    };
    let proposer = prod_key(3, "proposal_keypair").expect("proposer key");
    let slot = 11u32;
    let vote_root = [0x5au8; 32];
    let block_root = [0xb1u8; 32];

    // Interop: leanSig itself accepts ethean's natively produced signatures.
    let sigs: Vec<_> = voters.iter().map(|k| sign(k, slot, &vote_root)).collect();
    for (key, sig) in voters.iter().zip(&sigs) {
        let pk = xmss_public_key_from_ssz(key.public_key.as_bytes()).unwrap();
        let sig = xmss_signature_from_ssz(sig.as_bytes()).unwrap();
        assert!(xmss_verify(&pk, slot, &vote_root, &sig).is_ok());
    }

    let keys: Vec<PublicKey> = voters.iter().map(|k| k.public_key).collect();
    let raw: Vec<_> = keys.iter().copied().zip(sigs).collect();
    let started = std::time::Instant::now();
    let votes = aggregate_type1(&[], &raw, &vote_root, slot as u64).expect("type-1");
    eprintln!(
        "type-1 over 3 signatures: {:?}, {} bytes",
        started.elapsed(),
        votes.len()
    );

    let started = std::time::Instant::now();
    verify_single(&votes, &keys, &vote_root, slot as u64).expect("type-1 verifies");
    eprintln!("type-1 verify: {:?}", started.elapsed());
    assert_eq!(
        verify_single(&votes, &keys, &[0u8; 32], slot as u64),
        Err(MultisigError::BindingMismatch { component: 0 })
    );
    assert!(verify_single(&votes, &keys, &vote_root, slot as u64 + 1).is_err());
    assert!(verify_single(&votes, &keys[..2], &vote_root, slot as u64).is_err());

    let proposer_sig = sign(&proposer, slot, &block_root);
    let proposal = aggregate_type1(
        &[],
        &[(proposer.public_key, proposer_sig)],
        &block_root,
        slot as u64,
    )
    .expect("proposer type-1");

    let started = std::time::Instant::now();
    let block_proof = merge_type2(&[
        KeyedProof {
            public_keys: keys.clone(),
            proof: votes.clone(),
        },
        KeyedProof {
            public_keys: vec![proposer.public_key],
            proof: proposal,
        },
    ])
    .expect("type-2 merge");
    eprintln!(
        "type-2 merge: {:?}, {} bytes",
        started.elapsed(),
        block_proof.len()
    );

    let components = vec![
        ProofComponent {
            public_keys: keys.clone(),
            message: vote_root,
            slot: slot as u64,
        },
        ProofComponent {
            public_keys: vec![proposer.public_key],
            message: block_root,
            slot: slot as u64,
        },
    ];
    let started = std::time::Instant::now();
    verify_multi(&block_proof, &components).expect("type-2 verifies");
    eprintln!("type-2 verify: {:?}", started.elapsed());

    let mut swapped = components.clone();
    swapped.swap(0, 1);
    assert!(
        verify_multi(&block_proof, &swapped).is_err(),
        "order matters"
    );
    let mut wrong_proposer = components.clone();
    wrong_proposer[1].public_keys = vec![voters[0].public_key];
    assert!(verify_multi(&block_proof, &wrong_proposer).is_err());
    assert!(verify_multi(&block_proof, &components[..1]).is_err());

    let split = split_type2(
        &block_proof,
        &[keys.clone(), vec![proposer.public_key]],
        &vote_root,
    )
    .expect("split");
    verify_single(&split, &keys, &vote_root, slot as u64).expect("split type-1 verifies");
}

#[test]
fn hostile_bytes_are_rejected_without_panicking() {
    let Some(voters) = attesters(1) else {
        return;
    };
    let keys = vec![voters[0].public_key];
    let msg = [1u8; 32];
    let mut bomb = vec![0u8; 1024];
    bomb[..4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        verify_single(&bomb, &keys, &msg, 1),
        Err(MultisigError::DecompressionBound { .. })
    ));
    let mut garbage = vec![0xa5u8; 4096];
    garbage[..4].copy_from_slice(&8000u32.to_le_bytes());
    assert!(verify_single(&garbage, &keys, &msg, 1).is_err());
    let component = ProofComponent {
        public_keys: keys.clone(),
        message: msg,
        slot: 1,
    };
    assert!(verify_multi(&garbage, &[component]).is_err());
    assert!(verify_single(&[], &keys, &msg, 1).is_err());
    assert!(verify_single(&garbage, &[], &msg, 1).is_err());
}
