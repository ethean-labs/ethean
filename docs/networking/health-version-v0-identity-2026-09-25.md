# Health version + Hive v0 identity alias (2026-09-25)

## Why

1. `/lean/v0/health` returned only `status` + `service`; operators and Hive
   probes benefit from an explicit semver.
2. Identity was v1-only while ready/health already alias both prefixes.
3. `docker/hive/README.md` still claimed Lean HTTP was v1-only.

## What landed

- `HealthBody.version` = `CARGO_PKG_VERSION`.
- `GET /lean/v0/node/identity` aliases the identity handler.
- Hive README gap line updated for v0 + v1.

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v0/health
curl -s http://127.0.0.1:5052/lean/v0/node/identity
```
