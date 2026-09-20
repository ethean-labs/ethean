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

### Lean REST routes (`/lean/v1`, default `:5052`)

```bash
curl -s http://127.0.0.1:5052/lean/v1/health
curl -s http://127.0.0.1:5052/lean/v1/ready
curl -s http://127.0.0.1:5052/lean/v1/chain/head
curl -s http://127.0.0.1:5052/lean/v1/chain/finalized
curl -s http://127.0.0.1:5052/lean/v1/chain/sync
```

Route matching stays under `/lean/v1/…` only (no Beacon `/eth/v1`).
Disable with `--no-http`; Hive binds `0.0.0.0:5052`.

More route notes: [lean-http-api-5052-2026-09-20.md](../lean-http-api-5052-2026-09-20.md).
