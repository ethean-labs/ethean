# Admin event backlog gauge (2026-09-25)

## Why

Operators could poll `/lean/v*/events` for `pending`, but Grafana had no scrape
signal when the admin ring buffer was filling.

## What landed

- `ethean_admin_event_backlog` gauge (core family).
- Refreshed each metrics snap from `SharedApiState::events_pending()`.
- Node Health dashboard panel (version 5).

## Verify

```text
cargo test -p ethean-metrics --lib
cargo check -p ethean-node -q
curl -s http://127.0.0.1:9100/metrics | findstr admin_event_backlog
```
