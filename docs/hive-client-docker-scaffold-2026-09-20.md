# Hive client scaffold for Ethean (2026-09-20)

## What landed

Under `docker/hive/`:

| File | Role |
| --- | --- |
| `Dockerfile` | Multi-stage `cargo build -p ethean` → slim runtime |
| `ethean.sh` | Map `HIVE_*` Lean simulator env → `ethean start` |
| `client-devnet5.yaml` | Snippet to append in ethereum/hive |
| `README.md` | Build + honesty about gaps |

Image entrypoint uses `--until-signal --ephemeral` and network labels
`pq-devnet-5` / `pq-devnet-4` / `local`.

## Recipe

```bash
docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .
```

## Still open

| Gap | Notes |
| --- | --- |
| Upstream `clients/ethean` PR in ethereum/hive | Needs published image or git context |
| Consume Hive `config.yaml` / validators registry | CLI file flags |
| Fixed QUIC listen + Lean HTTP `:5052` | Interop with Hive RPC suites |
| A2/A3 / leanVM / bigint | External production gates |
