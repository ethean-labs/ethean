# SignedBlock assemble and gossip publish (2026-09-19)

## Assemble

- `assemble_signed_block` wraps a `PlanTransition` as `SignedBlock` (empty proof allowed)
- `encode_proposal_gossip` SSZ-encodes the envelope onto the Lean `/block/ssz_snappy` topic
- `ProposalGossip` carries topic, payload, block root, and `has_type2_proof`

## Duty step

- When `publish_allowed`: encode gossip, store on `ChainOwner.pending_block_gossip`
- Emits `ProposalGossipReady` alongside `ProposalPlanned`

## Network flush (`libp2p-quic`)

- `swarm_pump::publish_pending_block` Snappy-compresses and publishes via QuicSwarm
- Restores pending bytes if publish fails (retryable)
- `EtheanClient::flush_pending_block_gossip` is the caller-facing entry (in `client_swarm`)

## Verification

```text
cargo test -p ethean-node --lib block_builder::assemble
cargo test -p ethean-node --lib duty_step
cargo test -p ethean-node --features libp2p-quic --lib swarm_pump
cargo test -p ethean-node --features libp2p-quic --lib
```

## Still open

- Fold proposer sig + attestation aggregates into a real leanVM Type-2 envelope
- Production leanSig backend (`leansig-backend` feature) instead of test-hmac smoke keys
- leanVM FFI so non-empty Type-2 proofs verify on ingest
