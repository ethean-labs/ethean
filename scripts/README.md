# scripts

Developer and CI helper scripts for the Ethean Lean Consensus workspace.

Keep scripts English, non-interactive, and safe for `cargo` + bash on Linux.

## `ethean` on PATH (no install.sh)

`cargo build -p ethean` (debug or release) runs `bin/ethean/build.rs`, which writes a
shim into `$CARGO_HOME/bin` (usually `~/.cargo/bin`):

| Host | Shim |
| --- | --- |
| Linux | `ethean` (executable shell script) |

The shim prefers `target/release/ethean`, then `target/debug/ethean`, under this
workspace. After a successful build:

```bash
ethean version
ethean start --ticks 3
```

Rustup normally puts Cargo's `bin` on `PATH`. If the command is not found, add
`~/.cargo/bin` and open a new shell.

Do not use a separate `install.sh` / `cargo install` step for day-to-day runs.

## Version bump

Workspace version lives in root [`VERSION`](../VERSION) and
`[workspace.package] version` in `Cargo.toml`. After each development update:

```bash
./scripts/bump-version.sh
```

Patch goes `0.1.0` → `0.1.1` → … → `0.1.99` → `0.2.0`. See [docs/versioning.md](../docs/versioning.md).

## Network runners

| Script | Target | Notes |
| --- | --- | --- |
| `run-pq-devnet-4.sh` | **Operational default** | Long-run `--until-signal`, `/metrics` on `:9100` |
| `run-local-finality.sh` | **Solo finality** | Recent genesis, 4 validators, aggregator on |
| `run-pq-devnet-5.sh` | Ready path | Same binary; needs operator D5 multiaddrs |
| `local-pq-mesh.sh` | **Private mesh** | 2 peers; writes `target/local-pq-mesh/nodes.multiaddrs` then dials it |
| `run-observability.sh` | **Grafana + Prometheus** | Scrapes host `:9100`; UI on `:3000` / `:9090` |

```bash
./scripts/run-pq-devnet-4.sh
./scripts/run-pq-devnet-5.sh --bootnodes '/ip4/…/udp/…/quic-v1/p2p/…'
./scripts/local-pq-mesh.sh
./scripts/run-observability.sh
```

Grafana: http://localhost:3000 — dashboard **Ethean Lean Clients Dashboard**.
See [deploy/observability/README.md](../deploy/observability/README.md).

### Local private mesh (Ream-style)

There is no public pq-devnet bootnode list. Like Ream/ethlambda, exercise P2P by
creating a private mesh for this run:

1. Peer A starts with `--until-signal` and logs `dialable=…`
2. Script writes that multiaddr to `target/local-pq-mesh/nodes.multiaddrs`
   (Ethean equivalent of lean-quickstart `nodes.yaml`)
3. Peer B dials that file via `--bootnodes`

```bash
./scripts/local-pq-mesh.sh
NETWORK=pq-devnet-4 PEER_B_TICKS=15 ./scripts/local-pq-mesh.sh
```
