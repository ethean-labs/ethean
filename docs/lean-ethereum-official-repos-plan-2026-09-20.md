# leanEthereum official repos — plan and mapping (2026-09-20)

## Goal

Treat the five [leanEthereum](https://github.com/leanEthereum) tooling repos as
first-class sources of truth alongside leanroadmap generation cards, then fold
them into Ethean’s internal backlog.

## Local research (gitignored)

Deep notes live under `bazalinacaklar/lean-ethereum-official/` (not committed):

| Note | Repo |
| --- | --- |
| `leanSpec.md` | Protocol + Lstar fixtures / apitest |
| `leanSig.md` | XMSS hash-sig crate pin |
| `leanVM.md` | Aggregator zkVM prove/verify pin |
| `leanMetrics.md` | Prometheus `lean_*` catalog |
| `leanBench.md` | Cross-machine benches; D4/D5 API modes |
| `development-plan-2026-09-20.md` | Tracks A–E |

Index: `bazalinacaklar/lean-ethereum-official/README.md`.

## Authority order

1. leanroadmap.org generation (D4 Active / D5 Planned)
2. leanSpec pin + fixture digest
3. leanSig / leanVM SHAs for that generation (cross-check leanBench)
4. leanMetrics catalog for scrape interop
5. Peer clients = interop reference only

## Ethean gaps mapped

| Track | Status |
| --- | --- |
| A Spec/FC | STF green; FC 22+; `finalized_safety` orphan weights still open |
| B Crypto | Fail-closed; leanSig vendor patch; leanVM IPC mock ≠ live |
| C Metrics | `ethean_*` namespace; map to `lean_*`; add safe_target / reorg |
| D Bench | Align `LOG_INV_RATE` with leanBench prod default when live |
| E Mesh | Operator A2/A3 + Hive image (external) |

## This session follow-through

1. Research notes under `bazalinacaklar/lean-ethereum-official/` (done).
2. This tracked summary + README index link.
3. Engineering: expose `ethean_safe_target_slot` (leanMetrics
   `lean_safe_target_slot` analogue) and tighten FC weight ancestry if needed.

## Do not

Clone these repos into the git tree. Do not invent fork digests or bootnodes
from README text alone.
