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

Default `--network` is **`pq-devnet-4`** (operational join while D5 has no public mesh).

**`pq-devnet-5` stays fully wired** as a ready path: same CLI flags, config files under
`config/networks/pq-devnet-5.*`, and `scripts/run-pq-devnet-5.*`. Use
`--network pq-devnet-5` when operators publish multiaddrs.

That label alone does **not** attach to a public mesh. Supply QUIC multiaddrs:

| Source | pq-devnet-4 (default) | pq-devnet-5 (ready) |
| --- | --- | --- |
| CLI | `--bootnodes '…'` | same |
| Env | `ETHEAN_BOOTNODES=…` | same |
| File | `config/networks/pq-devnet-4.bootnodes` | `config/networks/pq-devnet-5.bootnodes` |
| Fork digest file | `config/networks/pq-devnet-4.forkdigest` | `config/networks/pq-devnet-5.forkdigest` |

Empty bootnodes → offline local duties with a clear warning log.

When eth/pq-devnets publish live digests, multiaddrs, and prover binaries, follow
the plug-in checklist (no fabricated bootnodes):
[pq-devnet-operator-plug-in-checklist-2026-09-20.md](pq-devnet/pq-devnet-operator-plug-in-checklist-2026-09-20.md).

Peer baseline: **Zeam** and **Ream** for current interop. **Peam** is older. **Beam** is historical naming only.

## Run

```bash
ethean version
ethean start
ethean start --ticks 3
ethean start --ticks 2 --wall-clock
ethean start --until-signal
ethean start --until-signal --network pq-devnet-4 --bootnodes '<quic-multiaddr>'
ethean start --until-signal --network pq-devnet-5 --bootnodes '<quic-multiaddr>'
ethean start --network local
ethean start --data-dir ./ethean-data
ethean validator
```

Helpers:

```bash
./scripts/run-pq-devnet-4.sh
./scripts/run-pq-devnet-5.sh   # ready path
```

`start` loads `lstar_devnet`, opens Lean in-memory storage (or `--data-dir` RocksDB when built with that feature), binds UDP/QUIC, smokes `/lean/v1/health`, dials bootnodes when present, then runs the duty loop.

## Monitor appearance

Terminal tracing only (network label, dials, crypto gates, ticks). No `ethean monitor` subcommand yet.

## Config / data

- Prefer paths and schema id from `ethean-storage` (`ethean-lc-d5-v1`).
- Refuse directories whose names contain legacy product markers (see storage `refuse_legacy_path`).
- HTTP surface is `/lean/v1/…` via `ethean-rpc` (no `/eth/v1` Beacon compatibility).

## Release tooling

See [release/README.md](./release/README.md). Working-client plan: [working-client-pq-devnet-5-plan-2026-09-19.md](pq-devnet/working-client-pq-devnet-5-plan-2026-09-19.md). Default-network note: [default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](pq-devnet/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md).
