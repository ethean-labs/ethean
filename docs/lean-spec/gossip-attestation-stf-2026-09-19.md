# Gossip attestation SSZ and structural STF (2026-09-19)

## Attestation decode

- `try_decode_attestation_root` for `/attestation_*/` topics
- Prefers `AggregatedAttestation`, then `SignedAggregatedAttestation`
- Content roots feed `last_gossip_root` (pool/duty wiring still open)

## Structural STF on gossip blocks

- `DecodedBlockGossip` carries the decoded `Block`
- `gossip_stf::import_decoded_block`:
  - no local state/profile → root-only tip advance
  - state + profile → `apply_block_unverified`; updates `head_state` + `head_root`
  - STF error with local state → reject (head unchanged)
- `ChainOwner.profile` set from client boot

## Verification

```text
cargo test -p ethean-node --lib gossip_decode
cargo test -p ethean-node --lib gossip_stf
cargo test -p ethean-node --lib dispatch
```

## Still open

- Attestation ingest into validator/aggregation pools
- Verified `apply_block(SignedBlock)` path when Type-2 proofs are present
- leanSig git dep / leanVM FFI
