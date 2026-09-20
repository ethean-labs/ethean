# Phase 07 — XMSS signer safety (2026-09-19)

## Summary

Phase 07 delivers the Lean XMSS crypto surface and a durable, role-separated signer.

- `ethean-crypto` (`crates/crypto`): PROD_CONFIG wire types (52-byte public key, 2536-byte signature, dimension 46, lifetime `2^32`). Production backend **fails closed** when `leansig-backend` is unavailable. Default `test-hmac` is a real MAC verify for tests (never always-true).
- `ethean-validator` (`crates/validator`): reservation → flush → sign → complete journal; Attestation/Proposal roles; idempotent retries; conflicting roots rejected; uncertain leaves burned.
- Node: deleted legacy `bls` / `wots` / local hash modules; crypto re-exports `ethean-crypto`.

## Workspace scaffolding (user request)

Root folders added: `artifacts/`, `scripts/`, `tools/`, and expanded `tests/{interop,security,recovery}/`.

Stub crates (compile, Phase 10–12 work later): `storage`, `network-wire`, `network`, `sync`, `rpc`, `metrics`.

## leanSig status

Pinned `15cbdd43ec8525aa43fea2f42cafc5ed366084ae`. Optional Cargo feature `leansig-backend` exists but did not resolve cleanly against the workspace lock (Plonky3 / `num_bigint`). Gate recorded in `spec/pins/phase-07.lock.toml`.

## Tests

- `cargo test -p ethean-crypto` — 12 passed
- `cargo test -p ethean-validator` — 5 passed

## Deferred

1. Production leanSig verify/sign once dependency graph resolves
2. Upstream XMSS fixture byte-diff
3. RocksDB-backed signer journal (Phase 11 storage)
4. Encrypted-at-rest key policy (release gate)
5. Aggregate proofs (Phase 08)
