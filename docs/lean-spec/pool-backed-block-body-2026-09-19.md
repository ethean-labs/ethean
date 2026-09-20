# Pool-backed block body selection (2026-09-19)

## Pool

- `PoolEntry.attestation_ssz` retains gossip `AggregatedAttestation` bytes
- Signed attestation gossip reconstructs bits + data before insert
- `AggregatePool::best_entries` returns one best-coverage entry per key, sorted by message root

## Block builder

- `block_builder::body_from_pool` packs a `BlockBody` from retained attestation SSZ (cap argument)
- Entries without attestation SSZ (aggregation-topic proofs only) are skipped

## Duty step

- On gate-pass ticks: prune pool before `max_head_lag_slots`, then call `body_from_pool` (proposal publish still deferred)

## SSZ fix (related)

- Bitlist decode included an off-by-one on the delimiter bit; trailing `true` data bits now round-trip

## Verification

```text
cargo test -p ethean-ssz --lib
cargo test -p ethean-node --lib
```

## Still open

- Sign / assemble Type-2 `SignedBlock` and gossip-publish when `publish_allowed`
- leanVM FFI so verified SignedBlock gossip can accept
