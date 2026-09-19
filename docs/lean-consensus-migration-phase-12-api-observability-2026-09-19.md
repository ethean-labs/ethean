# Phase 12 — API, observability, multiclient interop (2026-09-19)

## Summary

### ethean-rpc (8 tests)
- `/lean/v1` routes only; `/eth/v1` rejected
- Admin auth on non-loopback; body cap 256 KiB
- Redacted event backlog; Head/Finalized/Sync DTOs with trust_source

### ethean-metrics (6 tests)
- `ethean_` registry + Prometheus text export
- Forbidden high-cardinality labels
- Readiness requires storage/crypto/signer/network/prover

### deploy/observability
Prometheus scrape stub, Grafana provisioning stubs, alert rules (readiness, finality stall, prover timeout)

## Open
HTTP server/exporter bind in binary, Grafana image digests, Hive/mixed-client runs, OSD-009 leanMetrics pin
