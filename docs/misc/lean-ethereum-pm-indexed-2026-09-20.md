# leanEthereum/pm indexed (2026-09-20)

## Goal

Treat [leanEthereum/pm](https://github.com/leanEthereum/pm) as the meeting / high-level
devnet-plan source next to leanroadmap and the code repos (leanSpec, leanSig, …).

## What landed

### Local research (gitignored)

- `bazalinacaklar/lean-ethereum-official/pm.md`
- Index row in `lean-ethereum-official/README.md`
- Track **F** in `development-plan-2026-09-20.md`

### Authority note

| Source | Role |
| --- | --- |
| leanroadmap.org | Active / Planned generation card |
| pm `pq-interop` on **main** | Written high-level plans + pin tables |
| pm open PRs (#75 / #76) | Watch-only D5 drafts |

Main README still lists **pq-devnet-4 as Speccing**. Open PR #76 would mark D4
Completed and add D5 Speccing — do not flip Ethean defaults until merge +
leanroadmap agree.

### D4 plan takeaways kept

- Dual validator keys; recursive / in-block coalesce; `log_inv_rate` ∈ 1..=4
- Ethean keeps `MAX_ATTESTATIONS_DATA = 8` (leanSpec + D4 summary; objectives
  text that says 16 is not authoritative alone)
- `LOG_INV_RATE = 2` remains the production pin (inside D4 range)

## Follow-through engineering

- Document this index (this file).
- Strengthen FC `on_block` cap + crypto rate-range assert against D4/pm.
- D5 (block-level proof / Goldfish drafts) stays Planned scaffolding only.
