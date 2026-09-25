# Head-behind-finalized catch-up + local Hive smoke script (2026-09-25)

## Context

Hive sync still needs a live Docker re-run. This machine has a Docker **client**
but no running Desktop engine (`Docker Desktop.exe` not found; GHCR `:devnet5`
anonymous pull still fails). In-repo work continued on the recovery path and
operator tooling.

## What landed

| Piece | Change |
| --- | --- |
| `sync_catchup` | Keep peers whose **finalized** tip is ahead; when behind finalized, stage blocks-by-root for finalized (+ head) alongside deep range |
| Majority tip | Prefer ≥2 agreeing finalized tips over a lone ahead adversary — see [majority-status-tip-bad-checkpoint-2026-09-25.md](../networking/majority-status-tip-bad-checkpoint-2026-09-25.md) |
| `tools/hive/smoke-local.ps1` | Build local `ghcr.io/ethean-labs/ethean:devnet5`, apply drop-in, optional `--sim.limit sync` |
| `tools/hive/README.md` | Documents the local smoke path |

## Why finalized root pin

Hive “head-behind-finalized recovery” can stall if range responses are empty
while Status advertises a finalized tip ahead of local head. Fetching that
finalized root gives the node an anchor without inventing checkpoints.

## Operator next

1. Start Docker Desktop (or Linux daemon).
2. `.\tools\hive\smoke-local.ps1 -HiveRoot <hive-checkout>`
3. Flip GHCR package public + open ethereum/hive PR when `gh auth` is ready.

## Smoke (unit)

```bash
cargo test -p ethean-node --lib --features libp2p-quic -- sync_catchup::
```
