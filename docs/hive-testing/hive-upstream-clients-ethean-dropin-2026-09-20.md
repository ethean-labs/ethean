# Hive upstream `clients/ethean` drop-in (2026-09-20)

## What landed

In-repo drop-in for the ethereum/hive registration PR:

| Path | Role |
| --- | --- |
| `docker/hive/upstream-clients-ethean/Dockerfile` | Thin Hive client image wrapping `ghcr.io/ethean-labs/ethean:devnet5` |
| `docker/hive/upstream-clients-ethean/ethean.sh` | Strict entrypoint (requires prepared config + registry) |

Local build image remains `docker/hive/Dockerfile` → `ghcr.io/ethean-labs/ethean:local`.
The flexible local entrypoint (`docker/hive/ethean.sh`) stays optional for smoke
without Hive assets; the upstream script matches Ream’s fail-closed asset contract.

Current copy + PR checklist:
[`hive-quickstart-registration-2026-09-24.md`](./hive-quickstart-registration-2026-09-24.md).

## Operator steps (outside this repo)

1. `docker build -t ghcr.io/ethean-labs/ethean:devnet5 .` (or rely on GHCR)
2. Copy `docker/hive/upstream-clients-ethean/` → `ethereum/hive/clients/ethean/`
3. Append client rows + lean-devnets matrix (see registration note)
4. `./hive --sim lean --client-file simulators/lean/clients/devnet5.yaml --client ethean --docker.output --results-root ./workspace/logs`

## Still outside this repo

- Merged ethereum/hive PR + CI image publish
- Asset preparer client-id list if it hard-codes peer names
- Production `leansig-backend` Hive image variant
