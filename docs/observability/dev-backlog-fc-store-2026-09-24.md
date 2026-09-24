# Development backlog (2026-09-24)

## Current snapshot (v0.1.53)

- Live FC path + fork_choice HTTP + attest head≠target
- Admin `GET /lean/v1/events` JSON poll (redacted ring buffer)
- “External” backlog items documented (do not invent digests/bootnodes)

## What “external” means

See [what-external-backlog-means-2026-09-24.md](../process/what-external-backlog-means-2026-09-24.md).
Short form: **outside-repo ownership** (operator paste, leanSpec fill, Hive org).

## Backlog

| Pri | Area | Action |
| --- | --- | --- |
| P0–P1 | Live FC / attest / fork_choice API | **Done** |
| P1 | Admin events poll | **Done** |
| Ext | Fixture re-fill | leanSpec upstream fill (watch) |
| Ext | A2/A3 + Hive matrix | Paste / external PR only |

## This session

1. Document external vs in-repo — done.
2. Implement `/lean/v1/events` drain — done.
3. Version **0.1.53**.
