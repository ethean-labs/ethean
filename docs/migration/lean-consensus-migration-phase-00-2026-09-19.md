# Lean Consensus Migration — Phase 00 (2026-09-19)

## Summary

Phase 00 locked the Ethean protocol evidence set against **leanSpec `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8`** (tree `4a0a358a…`, fork **`lstar` / LstarSpec**). Tracked planning artifacts were added under `spec/` and `tests/interop/`; OSD-001..OSD-008 closed; OSD-009 and the Rust toolchain remain open. No product `src/` changes.

Ethean planning HEAD at phase start: **`f82bbbb179f04e742a2321fe4d50a76eb46c36be`** (recorded separately from the protocol pin).

## What was frozen

| Area | Selection |
| --- | --- |
| Fork choice / finality | Modified 3SF-era `lstar` (not Goldfish / PQ heartbeat) |
| `MAX_ATTESTATIONS_DATA` | 8 |
| Slot clock | 4s slot, 5 intervals |
| Registry / history | 4096 validators, `2^18` historical roots |
| XMSS | `PROD_CONFIG` Dim46 base8, Poseidon/KoalaBear; PK 52 B; sig 2536 B |
| Block proof | Type-2 `SignedBlock.proof = MultiMessageAggregate` |
| Aggregation | leanSpec XMSS + `lean-multisig-py` v0.0.9 → commit `39b9c397…` |
| Fixtures | `fixtures-prod-scheme.tar.gz` size 154123448, sha256 `21d9de70…` |
| Snappy | Gossip raw; req/resp framed |
| SSZ (Python ref) | `eth-ssz-specs>=0.1.0,<0.2` |

## Files written (tracked)

- `spec/README.md`
- `spec/pins/README.md`
- `spec/pins/phase-00.lock.toml`
- `spec/pins/protocol-surface.toml`
- `spec/fixtures/README.md`
- `spec/fixtures/phase-00/README.md`
- `spec/fixtures/phase-00/manifest.toml`
- `tests/interop/README.md`
- `road-to/lean-consensus-migration/02-protocol/OPEN_SPEC_DECISIONS.md` (updated)
- `road-to/lean-consensus-migration/02-protocol/COMPATIBILITY_LEDGER.md` (updated)
- `docs/lean-consensus-migration-phase-00-2026-09-19.md` (this file)

## Local notes (gitignored)

- `bazalinacaklar/lean-spec-snapshot.md`
- `bazalinacaklar/compatibility-matrix.md`

## Remaining blockers

1. **OSD-009 leanMetrics** — no separate git pin; leanSpec uses `prometheus-client`; keep unresolved or use `ethean_` namespace fallback until a pin exists.
2. **Rust toolchain** — `rustc` missing on the Phase 00 host; mark TBD; Phase 01+ must add `rust-toolchain.toml` without inventing a version here.
3. **Secondary opens** — gossip message-ID preimage order, fork identifier bytes, discovery, checkpoint trust, stateful signer durability, compatibility fingerprint.
4. **Fixture CI** — tarball not vendored; first CI job must download and verify digest before extract.
5. **pq-devnet plan commit** — still unresolved in the ledger (generation named by leanSpec tree, not a separate plan pin).

## Explicit non-actions

- No `src/` or product `Cargo.toml` identity changes.
- No Cursor plan file edits.
- No git commit from this phase handoff (parent/owner commits).
- No 154MB fixture archive in git.
