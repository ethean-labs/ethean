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
| `HIVE_HTTP_ENABLED` | `0` → `--no-http` (default on `:5052`) |
| `HIVE_HTTP_ADMIN_TOKEN` | `--http-admin-token` for admin routes |
| `HIVE_LISTEN_PORT` | `--listen-port` (default `9000`) |

Always runs `ethean start --until-signal --ephemeral` with Lean HTTP on `0.0.0.0:5052`
and QUIC on UDP `:9000` (override via `HIVE_LISTEN_PORT`).

## Upstream registration (not in this repo)

1. Add `clients/ethean/` in ethereum/hive (Dockerfile that pulls `ethpandaops/ethean` or builds from git).
2. Append to `simulators/lean/clients/devnet5.yaml`:

```yaml
- client: ethean
  nametag: devnet5
```

Snippet also kept as [`client-devnet5.yaml`](./client-devnet5.yaml).

## Known gaps vs Ream Hive client

- Prepared `config.yaml` / `validators.yaml` from the simulator are **ignored** until Ethean grows file-based network config flags.
- Lean HTTP serves `/lean/v1/…` only (no Beacon `/eth/v1`).
