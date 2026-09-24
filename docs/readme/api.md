# API Documentation

HTTP surfaces bound by the `ethean` binary today.

Short overview: [root README — API Documentation](../../README.md#api-documentation).

## Bound HTTP surfaces

### Health and metrics (bound today)

```bash
curl -s http://127.0.0.1:9100/healthz    # process up
curl -s http://127.0.0.1:9100/readyz    # subsystem gates
curl -s http://127.0.0.1:9100/metrics   # Prometheus exposition
```

### Lean REST (`/lean/v0` hive surface, default `:5052`)

Hive interop lives under `/lean/v0`. Matching `/lean/v1` paths are aliases. Beacon `/eth/` returns 404. JSON slots are numbers; roots are `0x` + lowercase hex. SSZ routes use `Content-Type: application/octet-stream`.

```bash
curl -s http://127.0.0.1:5052/lean/v0/health
curl -s http://127.0.0.1:5052/lean/v0/checkpoints/justified
curl -s http://127.0.0.1:5052/lean/v0/fork_choice
curl -s http://127.0.0.1:5052/lean/v0/states/finalized -o /tmp/state.ssz
curl -s http://127.0.0.1:5052/lean/v0/blocks/finalized -o /tmp/block.ssz
curl -s http://127.0.0.1:5052/lean/v0/admin/aggregator
curl -s -X POST http://127.0.0.1:5052/lean/v0/admin/aggregator \
  -H 'Content-Type: application/json' -d '{"enabled":true}'
```

Operator aliases still served:

```bash
curl -s http://127.0.0.1:5052/lean/v1/health
curl -s http://127.0.0.1:5052/lean/v1/ready
curl -s http://127.0.0.1:5052/lean/v1/chain/head
curl -s http://127.0.0.1:5052/lean/v1/chain/finalized
curl -s http://127.0.0.1:5052/lean/v1/chain/sync
```

Disable with `--no-http`; Hive binds `0.0.0.0:5052`. Chain views 503 until genesis is sealed.

Notes: [lean-http-api-5052-2026-09-20.md](../networking/lean-http-api-5052-2026-09-20.md),
[lean-v0-http-api-hive-2026-09-24.md](../networking/lean-v0-http-api-hive-2026-09-24.md).
