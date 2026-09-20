# Hive upstream `clients/ethean` drop-in (2026-09-20)

## What landed

In-repo drop-in for the ethereum/hive registration PR:

| Path | Role |
| --- | --- |
| `docker/hive/upstream-clients-ethean/Dockerfile` | Thin Hive client image wrapping `ethpandaops/ethean` |
| `docker/hive/upstream-clients-ethean/ethean.sh` | Strict entrypoint (requires prepared config + registry) |
| `docker/hive/upstream-clients-ethean/lean-devnets-snippet.txt` | `ethean=devnet4,devnet5` matrix line |
| `docker/hive/upstream-clients-ethean/README.md` | Copy + smoke checklist |

Local build image remains `docker/hive/Dockerfile` → `ethpandaops/ethean:local`.
The flexible local entrypoint (`docker/hive/ethean.sh`) stays optional for smoke
without Hive assets; the upstream script matches Ream’s fail-closed asset contract.

## Operator steps (outside this repo)

1. `docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .`
2. Copy `docker/hive/upstream-clients-ethean/` → `ethereum/hive/clients/ethean/`
3. Append client rows + lean-devnets matrix (see folder README)
4. `./hive --sim lean --client-file …/devnet5.yaml --client ethean --docker.output`

## Still outside this repo

- Merged ethereum/hive PR + CI image publish
- Asset preparer client-id list if it hard-codes peer names
- Production `leansig-backend` Hive image variant
