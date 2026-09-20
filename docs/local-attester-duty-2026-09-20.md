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
(XMSS sig as interim `proof` until leanVM Type-1 prove replaces it).

## Still open

- Propose only when `slot % n` matches an owned index
- Replace interim XMSS-as-proof with real leanVM Type-1 for gossip
- Upstream ethereum/hive `clients/ethean`
