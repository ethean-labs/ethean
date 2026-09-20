# Local attester duty wiring (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `LocalAttester` | Smoke HMAC + `from_key_record` (needs `leansig-backend`) |
| `ChainOwner` | `attester` + `owned_validator_indices` |
| `registry_apply` | Installs attestation key + stores owned indices |
| `duty_attest` | Interval-1 sign → single-bit aggregate into pool |
| `ChainEvent::AttestationSigned` | Observability for local votes |

On wall ticks with interval `1`, an installed attester signs attestation-data
`hash_tree_root` for the first owned validator index and seeds the aggregate pool
with **empty** `proof` (XMSS is never treated as Type-1). Same-tick aggregator
prove fills a real aggregate proof when available — see
[`attest-before-prove-empty-type1-2026-09-20.md`](../lean-crypto/attest-before-prove-empty-type1-2026-09-20.md).

## Still open

- Production leanVM Type-1 for gossip (IPC / FFI); smoke uses `test-aggregate`
- Upstream ethereum/hive `clients/ethean` merge (drop-in ready under `docker/hive/upstream-clients-ethean/`)
- Propose only when `slot % n` matches an owned index — **landed**
  (see [`owned-index-proposer-gate-2026-09-20.md`](owned-index-proposer-gate-2026-09-20.md))
