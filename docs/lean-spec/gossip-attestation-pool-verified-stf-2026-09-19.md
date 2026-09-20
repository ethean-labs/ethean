# Gossip attestation pool and verified SignedBlock STF (2026-09-19)

## Aggregate pool

- `AggregatePool::Default` now uses `new(8)` so a zero variant cap cannot empty keys on insert
- `ChainOwner.aggregates` retains gossip proofs keyed by transition agg-profile digest + message root
- `gossip_pool::ingest_into_pool` handles `/attestation_*/` and `/aggregation/` topics
- Unsigned aggregated attestations store coverage from bitlists (empty proof); signed envelopes keep proof bytes

## Decode

- `DecodedBlockGossip.signed` keeps the `SignedBlock` envelope when present
- `DecodedAttestationGossip` exposes `content_root`, `message_root`, `slot`, `coverage`, and `proof`

## STF

- Non-empty Type-2 proof on gossip → `apply_block(SignedBlock)` (fail closed on reject)
- Bare blocks / empty proofs → `apply_block_unverified` as before
- New outcome: `GossipStfResult::AppliedVerified`

## Dispatch

- `IngestGossip` always tries pool insert, then block import when the topic decodes as a block

## Verification

```text
cargo test -p ethean-node --lib gossip_decode
cargo test -p ethean-node --lib gossip_pool
cargo test -p ethean-node --lib gossip_stf
cargo test -p ethean-node --lib dispatch
```

## Still open

- Sign / assemble Type-2 `SignedBlock` and gossip-publish when `publish_allowed`
- Production leanVM FFI so verified proofs can accept (stub remains fail-closed)
- leanSig clean git dep (vendor patch path still required)
