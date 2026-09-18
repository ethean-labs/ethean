# Phase 03 fixtures

Provenance and expected cases for canonical SSZ and Lean types.

- Manifest: [manifest.toml](./manifest.toml)
- Lock: [`../../pins/phase-03.lock.toml`](../../pins/phase-03.lock.toml)
- Protocol surface: [`../../pins/protocol-surface.toml`](../../pins/protocol-surface.toml)

## Coverage (this phase)

Positive/negative unit tests live in `ethean-ssz` and `ethean-types` (Checkpoint, AttestationData, BlockHeader roundtrip + roots).

Large leanSpec production fixture tarball remains Phase 00 evidence (`fixtures-prod-scheme.tar.gz`); byte-level differential root vectors are not re-vendored here until the authoring host can run `cargo test`.

## Bounds under test

| Constant | Value |
| --- | --- |
| `MAX_ATTESTATIONS_DATA` | 8 |
| `VALIDATOR_REGISTRY_LIMIT` | 4096 |
| XMSS signature | 2536 bytes |
| Pubkey | 52 bytes |
