# External backlog explained + admin events poll (2026-09-24)

## What “external” means

Items labeled **external** are not blocked on missing Ethean code. They need
values or merges that live **outside this repository**:

| Item | Owner | Ethean stance |
| --- | --- | --- |
| A2 fork digest / A3 bootnodes | Live pq-devnet operators / session paste | Slots exist; **do not invent** |
| Fixture re-fill (`at_9` / `dead_9`) | leanEthereum/leanSpec `uv run fill` | Empty-body gates already; wait for fill |
| Hive `clients/ethean` matrix | ethereum/hive + lean-devnets | Drop-in shipped; registration is an external PR |

Full note: [what-external-backlog-means-2026-09-24.md](../process/what-external-backlog-means-2026-09-24.md).

## In-repo slice this session

While those stay external, implement the previously-501 admin events route:

- `SharedApiState` holds an `EventBuffer`
- `GET /lean/v1/events` drains redacted events as JSON (poll, not SSE yet)
- Duty mesh publishes `DutySuppressed` / `HeadUpdated`; snapshot publish emits
  `HeadSlot` / `Readiness`

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/events
```
