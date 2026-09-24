# Hive v0 alias for admin events (2026-09-25)

## Why

SSE/poll docs and the rpc README already advertised `/lean/v0/events`, but the
route table only matched `/lean/v1/events`. Hive surfaces live under `/lean/v0`.

## What landed

- `GET /lean/v0/events` aliases the same `AdminEvents` handler (JSON poll or SSE).
- Route tests cover both version prefixes.

## Verify

```text
cargo test -p ethean-rpc --lib routes::
curl -s http://127.0.0.1:5052/lean/v0/events
```
