# Start defaults to pq-devnet-5 network label (2026-09-19)

## Behaviour

- `ethean start` defaults `--network pq-devnet-5`.
- Bootnodes from `--bootnodes`, `ETHEAN_BOOTNODES`, or `config/networks/pq-devnet-5.bootnodes`.
- Empty list → WARN + offline local duties (verified with `cargo run -p ethean -- start --ticks 1`).
- Binary feature `libp2p-quic` is on by default; QuicSwarm binds and can dial when multiaddrs exist.

## Docs

- Root README Quick Start rewritten (removed Beacon/`Ethean monitor` fiction).
- `docs/deployment.md` + working-client plan updated.

## Still required for real mesh join

Operator multiaddrs + matching leanSpec fork digest + production leanSig/leanVM gates.
