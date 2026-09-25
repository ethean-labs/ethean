# Dependabot, GHCR public gate, Hive apitest honesty (2026-09-25)

## Why

Recommended queue after the external-gates tooling slice: Dependabot triage,
confirm `ghcr.io/ethean-labs/ethean:devnet5`, and keep the `:5052` hive surface
honest vs ethereum/hive `rpc_compat`. No inventing A2/A3 or fixture fill.

## What landed

| Piece | Change |
| --- | --- |
| `.github/dependabot.yml` | Weekly Cargo + GitHub Actions PRs (limit 5 each) |
| `.github/workflows/docker-image.yml` | Best-effort `PUT …/visibility` → public after manifest (continue-on-error) |
| `tools/hive/check-ghcr.ps1` | Anonymous pull probe for `:devnet5` / `:unstable` |
| `tools/hive/README.md` | Operator checklist: `gh auth`, GHCR public, apply, PR |
| `ethean-rpc` route tests | `hive_rpc_compat_v0_surface` locks the five hive paths |

## GHCR findings (this machine)

- Docker daemon was not running → no local `docker pull`.
- Anonymous GHCR token for `ethean-labs/ethean` failed → package **private**
  (or not readable). Hive wrappers cannot pull `:devnet5` until visibility is
  public (UI or a token that can `PUT` org package visibility).
- Docker image workflow on `master` already succeeds (amd64/arm64 + manifest)
  and tags `unstable` on master pushes; `:devnet5` only on `v*` tags or
  `workflow_dispatch` `extra_tags`.

## `:5052` honesty (hive `rpc_compat`)

Upstream `simulators/lean/src/scenarios/rpc_compat.rs` probes only:

- `GET /lean/v0/health`
- `GET /lean/v0/checkpoints/justified`
- `GET /lean/v0/fork_choice`
- `GET /lean/v0/states/finalized`
- `GET /lean/v0/blocks/finalized`

Operator extras (`/lean/v1/chain/*`, duties, SSE events, ready/identity) are
supported but **not** hive rpc_compat requirements. `GET /metrics` on `:5052`
stays a 404 pointer to `:9100` (no fake scrape).

## Still blocked externally

1. `gh auth login` → open ethereum/hive PR (ethean still absent from
   `simulators/lean/clients/devnet5.yaml` upstream).
2. Flip GHCR package public + ensure `:devnet5` tag exists.
3. A2/A3 paste; leanSpec `fill` for `at_9` / `dead_9`.

## Smoke

```powershell
.\tools\hive\check-ghcr.ps1
cargo test -p ethean-rpc --lib -- hive_rpc_compat_v0_surface
```
