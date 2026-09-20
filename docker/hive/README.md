# Ethean Hive client scaffold

Local Docker assets for registering Ethean in [ethereum/hive](https://github.com/ethereum/hive)
`simulators/lean`.

## Build

From the repository root:

```bash
docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .
```

## Entrypoint env (Lean simulator)

| Variable | Mapping |
| --- | --- |
| `HIVE_LEAN_DEVNET_LABEL` | `devnet5` → `--network pq-devnet-5` |
| `HIVE_BOOTNODES` | `--bootnodes` (omit when empty/`none`) |
| `HIVE_FORK_DIGEST` / `HIVE_LEAN_FORK_DIGEST` | `--fork-digest` |
| `HIVE_VALIDATORS` | `--validators` |
| `HIVE_IS_AGGREGATOR` | `0` → `--no-aggregator` |
| `HIVE_METRICS_ENABLED` | `1` keeps scrape on `:9100` |

Always runs `ethean start --until-signal --ephemeral`.

## Upstream registration (not in this repo)

1. Add `clients/ethean/` in ethereum/hive (Dockerfile that pulls `ethpandaops/ethean` or builds from git).
2. Append to `simulators/lean/clients/devnet5.yaml`:

```yaml
- client: ethean
  nametag: devnet5
```

Snippet also kept as [`client-devnet5.yaml`](./client-devnet5.yaml).

## Known gaps vs Ream Hive client

- No Beacon-style HTTP on `:5052` yet (Hive RPC suites will skip or fail).
- QUIC listen port is OS-assigned (`listen_port: 0`), not fixed `:9000`.
- Prepared `config.yaml` / `validators.yaml` from the simulator are **ignored** until Ethean grows file-based network config flags.
