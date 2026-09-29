# Prover timing, block data cap and verifier warm-up (2026-09-29)

## Measurement

New ignored probe `bin/ethean-prover/tests/timing.rs` drives the real
`ethean-prover` process. It generates PROD keys in-process and cycles them to
fill larger raw sets. Settings: release build, leanVM `e2592df4`, 20-core
Windows dev machine, two runs a few minutes apart. Run 1 was consistently
slower (cause not identified); treat the spread as host noise.

| Step | Run 1 | Run 2 |
| --- | --- | --- |
| Cold prover start + first Type-1 | 10.1 s | 5.5 s |
| Type-1, 1 raw signature | 0.56 s | 0.42 s |
| Type-1, 8 / 32 / 64 / 128 raw | 1.1–1.7 s | 0.45–0.54 s |
| First in-process Type-1 verify (verifier setup) | 10.2 s | 5.2 s |
| Warm Type-1 verify | 50–90 ms | 30 ms |
| Type-2 merge, 1 component | 3.6 s | 1.7 s |
| Type-2 merge, 2 components | 3.3 s | 3.3 s |
| Type-2 merge, 3 components | — | 5.4 s |
| Type-2 merge, 4 components | 5.8 s | 5.8 s |
| Type-2 merge, 8 components | 11.5 s | — |

Findings:

- **Type-1 barely depends on the raw count** (1 → 128 signatures), and proof
  size stays near 180–190 KiB. The old "0.33 s" figure came from a 2–3
  signature run; 0.5 s is the better planning number.
- **Type-2 merge steps with the next power of two of the component count**:
  1 → 1.7 s, 2 → 3.3 s, 3–4 → 5.6 s, 5–8 → 11.5 s. A block merges one
  component per attestation data plus the proposer's. At the spec maximum of
  8 data (9 components) the next step is 16, so a full block cannot be proved
  inside a 4 s slot. `accept_block_proof` then drops the block because the head
  has moved.
- **Verifier setup costs 5–10 s on first use.** The node never warmed it, so
  the first gossiped block or aggregate stalled the chain-owner loop for that
  long.

## Changes

- `--max-block-attestation-data N` (0–8, see
  `block_builder::DEFAULT_MAX_BLOCK_ATTESTATION_DATA`):
  - The default was 3 at first and is now 1; see
    [safe-target-merge-block-data-cap-1-2026-09-29.md](../lean-spec/safe-target-merge-block-data-cap-1-2026-09-29.md)
    for the devnet runs behind the change.
  - The cap is applied inside leanSpec selection (`select_body_capped`), which
    skips votes that change nothing when capped; see
    [mesh-finality-fixes-2026-09-29.md](../lean-spec/mesh-finality-fixes-2026-09-29.md).
  - Solo local-finality blocks carry no proof and keep the spec maximum.
  - Cap 3 keeps the merge at four components (~5.6 s) instead of 16. Cap 1
    keeps it at two (~3.3 s).
- The verifier is warmed at `start_with` on a `lean-verifier-setup` thread
  whenever local finality is off, and it logs `leanMultisig verifier ready` with
  its time.

## Block timeline by data count (run 2 numbers)

The early build starts at interval 4 of slot N (3.2 s). The proposer Type-1
takes about 0.5 s, then the merge:

| Data in block | Components | Block ready at |
| --- | --- | --- |
| 0 | 1 | ~5.4 s (interval 2 of N+1) |
| 1 | 2 | ~7.0 s (interval 4 of N+1) |
| 2–3 | 3–4 | ~9.3 s (slot N+2) |

So even capped, multi-data blocks land late on this machine. The cap bounds the
damage; it does not make 4 s slots comfortable at this leanVM pin. Devnet
hardware numbers still need to be collected, which requires a live mesh.

```bash
cargo test --release -p ethean-prover --test timing -- --ignored --nocapture
# env: ETHEAN_PROBE_KEYS=2 ETHEAN_PROBE_SIZES=1,8 ETHEAN_PROBE_COMPONENTS=1,2,3,4
```
