# Development backlog (2026-09-25)

## Current snapshot (v0.1.67)

- Late-joiner sync gate + range follow-up landed
  (`docs/networking/late-joiner-sync-catchup-2026-09-25.md`).
- External gates: Hive apply tooling shipped; A2/A3 + fixture fill still blocked
  on upstream publish / leanSpec fill (do not invent).
- Next in-repo: Hive sync suite re-verify; same-client finality / genesis state root.
- Day-to-day remote: `ethean-labs/ethean`

## External status

| Gate | Status |
| --- | --- |
| A2/A3 | Blocked — no public digest/bootnodes on leanroadmap |
| Fixture re-fill | Blocked — leanSpec `uv run fill` ownership |
| Hive matrix | Drop-in + `tools/hive/apply-ethean-client.*` ready; needs ethereum/hive PR |
| GHCR `:devnet5` public | Anonymous pull still private (operator flip) |
