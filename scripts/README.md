# scripts

Developer and CI helper scripts for the Ethean Lean Consensus workspace.

Keep scripts English, non-interactive, and safe for `cargo` + PowerShell on Windows.

## `ethean` on PATH (no install.sh)

`cargo build -p ethean` (debug or release) runs `bin/ethean/build.rs`, which writes a
shim into `$CARGO_HOME/bin` (usually `~/.cargo/bin`):

| Host | Shim |
| --- | --- |
| Windows | `ethean.cmd` |
| Linux / macOS | `ethean` (executable shell script) |

The shim prefers `target/release/ethean`, then `target/debug/ethean`, under this
workspace. After a successful build:

```bash
ethean version
ethean start --ticks 3
```

Rustup normally puts Cargo's `bin` on `PATH`. If the command is not found, add
`~/.cargo/bin` (Windows: `%USERPROFILE%\.cargo\bin`) and open a new shell.

Do not use a separate `install.sh` / `cargo install` step for day-to-day runs.

## Network runners

| Script | Target | Notes |
| --- | --- | --- |
| `run-pq-devnet-4.sh` / `.ps1` | **Operational default** | Long-run `--until-signal`, `/metrics` on `:9100` |
| `run-pq-devnet-5.sh` / `.ps1` | Ready path | Same binary; needs operator D5 multiaddrs |
| `local-pq-mesh.sh` / `.ps1` | **Private mesh** | 2 peers; writes `target/local-pq-mesh/nodes.multiaddrs` then dials it |
| `run-observability.sh` / `.ps1` | **Grafana + Prometheus** | Scrapes host `:9100`; UI on `:3000` / `:9090` |

```bash
# Unix
./scripts/run-pq-devnet-4.sh
./scripts/run-pq-devnet-5.sh --bootnodes '/ip4/…/udp/…/quic-v1/p2p/…'
./scripts/local-pq-mesh.sh

# Windows PowerShell
.\scripts\run-pq-devnet-4.ps1
.\scripts\run-pq-devnet-5.ps1 --bootnodes '/ip4/…/udp/…/quic-v1/p2p/…'
.\scripts\local-pq-mesh.ps1
.\scripts\run-observability.ps1
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

```powershell
.\scripts\local-pq-mesh.ps1
.\scripts\local-pq-mesh.ps1 -Network pq-devnet-4 -PeerBTicks 15
```
