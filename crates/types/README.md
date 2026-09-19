# ethean-types

Lean Consensus domain containers aligned to leanSpec **lstar** (`0b7d33ec`).

## Naming

Lean names only: `Block`, `SignedBlock`, `State`, `Checkpoint` (slot-based), dual XMSS `Validator` keys. No `BeaconBlock`, `BeaconState`, or `ExecutionPayload`.

## Constants

From `spec/pins/protocol-surface.toml` / `ethean-profile`:

| Constant | Value |
| --- | --- |
| `VALIDATOR_REGISTRY_LIMIT` | 4096 |
| `HISTORICAL_ROOTS_LIMIT` | 262144 |
| `MAX_ATTESTATIONS_DATA` | 8 |
| XMSS pubkey | 52 bytes |
| XMSS signature | 2536 bytes |
| Aggregate proof byte list | 512 KiB |

`AggregatedAttestations` SSZ list limit follows leanSpec (`VALIDATOR_REGISTRY_LIMIT`). `MAX_ATTESTATIONS_DATA` is the distinct attestation-data consensus cap.

## SSZ

Roots and bytes go through `ethean-ssz` (Ethean-owned). Python leanSpec uses `eth-ssz-specs`.
