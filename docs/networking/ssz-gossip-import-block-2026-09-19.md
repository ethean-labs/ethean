# SSZ gossip decode into ImportBlock (2026-09-19)

## Decode

- `gossip_decode::try_decode_block` accepts `/block/` topics as `SignedBlock` or bare `Block`
- Content root prefers `hash_tree_root`; aggregation topics use `MultiMessageAggregate`
- Unknown topics keep the provisional SHA-256 domain hash

## Dispatch

- `IngestGossip` sets `last_gossip_root`
- When a decoded block's `parent_root` equals the current head, head advances (same rules as `ImportBlock`)
- Non-matching parents still record the gossip root without moving head

## Verification

```text
cargo test -p ethean-node --lib gossip_decode
cargo test -p ethean-node --lib dispatch
pwsh tools/release/check-leansig-vendor.ps1
```

## Still open

- Feeding pool best-coverage into block builder / proposer duties
- leanSig git dep / leanVM FFI (verified path fails closed until FFI links)
