# Deployment (Lean Consensus)

Ethean Lean Consensus Client binary: `ethean` (`bin/ethean`).

## Build

```bash
cargo build -p ethean --release
```

Requires the workspace Rust toolchain (see `rust-toolchain.toml`). The binary enables `libp2p-quic` by default so bootnode dials work.

Each build of `bin/ethean` refreshes a PATH shim in `~/.cargo/bin` (`ethean` on
Unix, `ethean.cmd` on Windows) so the client runs as `ethean …` with no
`install.sh` / `cargo install` step. Cargo's `bin` directory must be on `PATH`
(rustup usually configures this).

## Network target

Default `--network` is **`pq-devnet-5`** (leanroadmap generation: planned / in progress).
That label alone does **not** attach to a public mesh. Supply QUIC multiaddrs:

| Source | Example |
| --- | --- |
| CLI | `--bootnodes '/ip4/…/udp/…/quic-v1/p2p/…'` |
| Env | `ETHEAN_BOOTNODES=…` |
| File | `config/networks/pq-devnet-5.bootnodes` |

Empty bootnodes → offline local duties with a clear warning log.

Peer baseline for this generation: **Zeam** and **Ream** (static bootnodes / `devnet5`). **Peam** is older. **Beam** is historical naming only.

## Run

```bash
ethean version
ethean start
ethean start --ticks 3
ethean start --ticks 2 --wall-clock
ethean start --until-signal
ethean start --until-signal --bootnodes '<quic-multiaddr>'
ethean start --network local
ethean start --data-dir ./ethean-data
ethean validator
```
`start` loads `lstar_devnet`, opens Lean in-memory storage (or `--data-dir` RocksDB when built with that feature), binds UDP/QUIC, smokes `/lean/v1/health`, dials bootnodes when present, then runs the duty loop.

## Monitor appearance

Terminal tracing only (network label, dials, crypto gates, ticks). No `ethean monitor` subcommand yet.

## Config / data

- Prefer paths and schema id from `ethean-storage` (`ethean-lc-d5-v1`).
- Refuse directories whose names contain legacy product markers (see storage `refuse_legacy_path`).
- HTTP surface is `/lean/v1/…` via `ethean-rpc` (no `/eth/v1` Beacon compatibility).

## Release tooling

See [release/README.md](./release/README.md). Working-client plan: [working-client-pq-devnet-5-plan-2026-09-19.md](./working-client-pq-devnet-5-plan-2026-09-19.md).
