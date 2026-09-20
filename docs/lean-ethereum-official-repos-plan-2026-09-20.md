# leanEthereum official repos — plan and mapping (2026-09-20)

## Goal

Treat the [leanEthereum](https://github.com/leanEthereum) tooling and PM repos as
first-class sources of truth alongside leanroadmap generation cards, then fold
them into Ethean’s internal backlog.

## Local research (gitignored)

Deep notes live under `bazalinacaklar/lean-ethereum-official/` (not committed):

| Note | Repo |
| --- | --- |
| `pm.md` | [leanEthereum/pm](https://github.com/leanEthereum/pm) — meetings + pq-devnet plans |
| `leanSpec.md` | Protocol + Lstar fixtures / apitest |
| `leanSig.md` | XMSS hash-sig crate pin |
| `leanVM.md` | Aggregator zkVM prove/verify pin |
| `leanMetrics.md` | Prometheus `lean_*` catalog |
| `leanBench.md` | Cross-machine benches; D4/D5 API modes |
| `development-plan-2026-09-20.md` | Tracks A–F |

Index: `bazalinacaklar/lean-ethereum-official/README.md`.

## Authority order

1. leanroadmap.org generation (D4 Active / D5 Planned)
2. leanEthereum/pm pq-interop plans **on main** (unmerged PRs = watch-only)
3. leanSpec pin + fixture digest
4. leanSig / leanVM SHAs for that generation (cross-check leanBench)
5. leanMetrics catalog for scrape interop
6. Peer clients = interop reference only

## Ethean gaps mapped

| Track | Status |
| --- | --- |
| A Spec/FC | STF green; FC 23+; `finalized_safety` still open |
| B Crypto | Fail-closed; `LOG_INV_RATE=2` in D4 range 1..=4 |
| C Metrics | `ethean_*` + safe_target / fc_reorg series |
| D Bench | Align comments with leanBench when live |
| E Mesh | Operator A2/A3 + Hive (external) |
| F PM | Indexed; await #76 merge before D5 status flip |
| E Mesh | Operator A2/A3 + Hive image (external) |

## This session follow-through

1. Research notes under `bazalinacaklar/lean-ethereum-official/` (done).
2. This tracked summary + README index link.
3. Engineering: expose `ethean_safe_target_slot` (leanMetrics
   `lean_safe_target_slot` analogue) and tighten FC weight ancestry if needed.

## Do not

Clone these repos into the git tree. Do not invent fork digests or bootnodes
from README text alone.
