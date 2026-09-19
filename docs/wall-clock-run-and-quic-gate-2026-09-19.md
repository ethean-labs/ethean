# Wall-clock run mode and QUIC multiaddr gate (2026-09-19)

## CLI / client

- `StartConfig` / `RunMode`: `SmokeElapsed` vs `WallClock`
- `run_wall_duty_loop`: system clock ticks; optional `tokio` sleep to next interval
- `ethean start --ticks N` and `--wall-clock`
- `EtheanClient::start_with` selects the mode; `start()` keeps smoke defaults

## Network

- `reject_non_quic`: refuse `/tcp`, `/ws`, `/wss`
- `prepare_transport`: empty fingerprint / port 0 refused, then `TransportPending`

## Verification

```text
cargo test -p ethean-node -p ethean-network --lib
cargo check -p ethean
```

ethean-node 18 tests, ethean-network 10 tests.

## Still open

libp2p QUIC-v1 swarm bind, RocksDB open implementation, leanSig/leanVM FFI, continuous until-signal loop (Ctrl-C).
