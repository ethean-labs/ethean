# Console log verbosity (2026-09-20)

Default `ethean start` is quiet: **INFO** for Ethean, **WARN** for libp2p /
gossipsub (no heartbeat spam).

| Flag | Effect |
| --- | --- |
| (default) | INFO + quiet libp2p |
| `-v` / `--verbose` | Ethean DEBUG; libp2p still WARN |
| `-vv` | DEBUG including libp2p |
| `-vvv` | TRACE |
| `--log-level LEVEL` | Set max level (`info`/`debug`/`trace`/…) |
| `RUST_LOG=…` | Full override when set |

Examples:

```text
ethean start --until-signal --network pq-devnet-4 --data-dir test-devnet
ethean start --until-signal --network pq-devnet-4 --data-dir test-devnet -v
ethean start --until-signal --network pq-devnet-4 --data-dir test-devnet -vv
```

Code: `bin/ethean/src/log_filter.rs`, `bin/ethean/src/file_log.rs`.
