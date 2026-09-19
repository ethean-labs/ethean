# Local finality for solo long-run (2026-09-20)

## Problem

`ethean start --until-signal --network pq-devnet-5` sat for hours with
`head_slot=0` / `finalized_slot=0` because:

1. Genesis time was `1_700_000_000` → wall slot in the millions; first proposal
   tried to `process_slots` across that gap (appears hung).
2. `head_root` stayed zero → parent mismatch on proposals.
3. Published blocks were never self-applied → solo head never moved.
4. No public bootnodes (expected) and no aggregator/local-finality path.

## Fix

- Recent genesis + `--validators N` (default 4)
- Seal genesis `head_root` (cache header `state_root` first)
- `--local-finality` (default on): inject full-registry attestation, self-apply
- `--is-aggregator` default on (`--no-aggregator` / `--no-local-finality` to off)
- Script: `scripts/run-local-finality.*`

## Cursor rule

`.cursor/rules/12-no-research-on-github.mdc` — research under `bazalinacaklar/`
and peer clones must never be committed (gitignore strengthened).

## Run

```powershell
ethean start --until-signal --network pq-devnet-4 --metrics
# or
.\scripts\run-local-finality.ps1 -MetricsStack
```

Watch http://127.0.0.1:9100/metrics for `ethean_head_slot` climbing.

Empty-mesh gossip publish (`InsufficientPeers`) is a soft skip under local
finality so the duty loop stays up — see
[local-finality-insufficient-peers-soft-skip-2026-09-20.md](local-finality-insufficient-peers-soft-skip-2026-09-20.md).

Grafana still needs a healthy Docker Desktop/WSL; without it use `:9100` only —
[docker-desktop-wsl-execerror-2026-09-20.md](docker-desktop-wsl-execerror-2026-09-20.md).
