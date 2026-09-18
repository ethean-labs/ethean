# Phase 03: Canonical SSZ and Lean types (2026-09-19)

## Summary

Introduced `ethean-ssz` and `ethean-types`, wired them into the workspace, deleted `crates/node/src/types/` Beacon containers, and pointed node storage/network/consensus consumers at Lean names (`Block`, `SignedBlock`, `State`, slot-based `Checkpoint`, dual XMSS `Validator`).

## Crates

### `crates/ssz` (`ethean-ssz`)

Minimal SSZ: LE integers, bool, fixed bytes, offset lists, bitlists, SHA-256 `hash_tree_root` (chunk packing / merkleize / mix-in-length). Ethean-owned; no peer codec crates. Phase 00 Python leanSpec still cites `eth-ssz-specs`.

### `crates/types` (`ethean-types`)

Mirrors leanSpec lstar containers at `0b7d33ec`:

- `Checkpoint { root, slot }`, `AttestationData`
- `Validator` with `Bytes52` attestation + proposal keys
- `BlockHeader`, `BlockBody`, `Block`, `SignedBlock` + `MultiMessageAggregate` stub
- `State` with genesis config, justified/finalized checkpoints, historical hashes, validators, justification bitlists (helpers TODO)
- Operations: `Attestation`, `SignedAttestation` (2536-byte sig check), `AggregatedAttestation`, aggregates

Bounds: `VALIDATOR_REGISTRY_LIMIT=4096`, `HISTORICAL_ROOTS_LIMIT=262144`, `MAX_ATTESTATIONS_DATA=8` (distinct-data consensus cap; AggregatedAttestations SSZ list limit remains 4096 per leanSpec).

`hash_tree_root` implemented for Checkpoint, AttestationData, BlockHeader (plus broader tree helpers).

## Node migration

- Deleted entire `crates/node/src/types/` tree (BeaconBlock/BeaconState/ExecutionPayload/…).
- Imports use `ethean_types::*`.
- Storage persists Block/State via SSZ bytes (not JSON roots).
- Checkpoint storage metadata keyed by **slot**.
- Consensus modules (transition, fork choice, slashing, attestation/block processing, validator economics) reduced to compile-stable stubs for Phase 05 — no Beacon type names remain in those surfaces.
- Network `NetworkMessage` no longer Serde-encodes Lean containers; status uses `finalized_slot`.

## Artifacts

- `spec/pins/phase-03.lock.toml`
- `spec/fixtures/phase-03/manifest.toml` + README
- Local note: `bazalinacaklar/phase-03-ssz-types.md` (gitignored)

## Remaining compile / interop risks

1. **Authoring host has no rustc** — workspace coherence is by inspection; run `cargo test -p ethean-ssz -p ethean-types` when toolchain exists.
2. **`State::ssz_decode`** is incomplete (encode + `hash_tree_root` present); full offset decode deferred.
3. **No differential leanSpec root vectors** vendored yet — unit roots are self-consistency only.
4. **Consensus stubs** will panic/`Err(Stub)` if exercised; Phase 05 must replace them.
5. **API DTO names** still contain transitional Beacon wording in HTTP schemas (`BeaconCommitteeSubscription`, etc.); consensus path is Lean-named.
6. **List merkleization limit** for AggregatedAttestations is 4096; enforce `MAX_ATTESTATIONS_DATA=8` at transition validation, not only at SSZ list capacity.
7. **serde on primitives** still used by API/WebSocket for Slot/Epoch; consensus containers intentionally lack Serde.

## Pin

leanSpec `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` (lstar), `MAX_ATTESTATIONS_DATA=8`.
