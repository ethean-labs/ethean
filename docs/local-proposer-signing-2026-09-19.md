# Local proposer signing on duty ticks (2026-09-19)

## Signer

- `LocalProposer` wraps `ethean_validator::run_proposer` on `TestHmacBackend`
- Smoke key: proposal role, activation slot 0, 1_000_000 active slots
- Wired onto `ChainOwner.proposer` (optional) and loaded in `EtheanClient::with_genesis_store`

## Plan / gossip

- `PlanTransition.proposer_signature` holds the wire signature bytes
- Gossip `SignedBlock` Type-2 field stays pool-only (empty unless pool had a proof)
- Raw XMSS/HMAC is **not** stuffed into Type-2 (would fail-closed on remote `apply_block`)
- `ProposalGossip.proposer_sig_len` and `ProposalGossipReady.proposer_sig_len` expose binding size

## Events

- `ProposalSigned { root, signature_len }` after a successful local sign
- Then `ProposalPlanned` / `ProposalGossipReady` as before

## Verification

```text
cargo test -p ethean-node --lib local_proposer
cargo test -p ethean-node --lib duty_step
cargo test -p ethean-node --lib
```

## Still open

- Fold proposer sig + attestation aggregates into a real leanVM Type-2 envelope
- Production leanSig backend (`leansig-backend` feature) instead of test-hmac smoke keys
- leanVM FFI so non-empty Type-2 proofs verify on ingest
