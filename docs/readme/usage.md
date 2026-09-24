# Usage

How to build, run, and operate Ethean locally (pq-devnet labels, dual mode, mesh, metrics, configuration, and troubleshooting).

Short overview: [root README — Usage](../../README.md#usage).

## Quick Start

Binary name is `ethean` (crate `ethean`).

**Operational default:** `--network pq-devnet-4` (D5 has no public always-on mesh yet).
**Ready path kept:** `--network pq-devnet-5` plus `config/networks/pq-devnet-5.bootnodes` /
`pq-devnet-5.forkdigest` and `scripts/run-pq-devnet-5.*` — flip when operators publish multiaddrs.

Without bootnodes (`--bootnodes` / `ETHEAN_BOOTNODES` / the matching
`config/networks/<label>.bootnodes` file), the node runs **offline** under that label
(local smoke duties).

### Build

```bash
cargo build -p ethean --release
```

After that build, the command is `ethean`. The build script
places a shim in `~/.cargo/bin`; no `install.sh` step.

### Run (default: pq-devnet-4, local finality on)

Solo long-run advances **head / justified / finalized** without public bootnodes
(4 validators, aggregator + local self-apply). Use `--until-signal`
so the process stays up for Grafana.

Ethean keeps **two local modes** (see the dual-mode section below): ephemeral
smoke, or peer-like fixed genesis under `--data-dir`.

```bash
ethean version
# Ephemeral smoke (new recent genesis every process start)
ethean start --until-signal --network pq-devnet-4 --ephemeral
# Peer-like durable (fixed genesis pin + head resume)
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data
ethean start --until-signal --network pq-devnet-5 --validators 4 --data-dir ./ethean-data
# Short smoke (always ephemeral)
ethean start --ticks 3
ethean start --ticks 2 --wall-clock
ethean start --network local --ephemeral
# Opt out of local finality / aggregator if needed
ethean start --until-signal --no-local-finality --no-aggregator --ephemeral
```

Helpers (build + long-run):

```bash
./scripts/run-pq-devnet-4.sh
./scripts/run-local-finality.sh          # same defaults; NETWORK=… VALIDATORS=…
METRICS_STACK=1 ./scripts/run-local-finality.sh
```

Paste D4 QUIC multiaddrs into `config/networks/pq-devnet-4.bootnodes` (or pass
`--bootnodes` / `ETHEAN_BOOTNODES`) before expecting a live mesh dial.

### Dual mode: peer-like persist vs ephemeral smoke

Ideal local setup keeps **both** paths:

| Mode | Flags | Behavior |
| --- | --- | --- |
| **Durable** | `--data-dir ./ethean-data` | Writes `genesis.json` / `genesis.ssz`, `state.ssz`, `blocks/*.ssz`, and `ethean.redb`; Ctrl-C then restart **continues** from last head / justified / finalized |
| **Ephemeral smoke** | `--ephemeral` (or omit `--data-dir`) | Builds a **new recent genesis** each start; slot counters reset (e.g. stop at 48 → next run near 4–5) |

```bash
# Durable long-run (recommended when you care about resume)
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data

# Stop with Ctrl-C, then restart — same genesis, resumed head
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data

# Wipe the data-dir (chain + logs) and start a new fixed chain
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data --reset-chain

# Fast solo smoke (no disk chain identity)
ethean start --until-signal --network pq-devnet-4 --ephemeral
```

Under `--data-dir` the node writes:

- `genesis.json` — Ethean operator bundle (`ethean-genesis-v1`: profile, keys, full genesis state)
- `genesis.ssz` — SSZ-encoded genesis `State`
- `state.ssz` / `head.root` — current head (updated each duty step)
- `blocks/<root>.ssz` — signed block blobs when a proposal is in flight
- `ethean.redb` — same SSZ blobs in a local KV (`historical_block_hashes` lives inside state SSZ)
- `log/ethean-YYYY-MM-DD-HHMMSS-log` — process log for that start (console still prints in green-forward ANSI; file is plain; wiped by `--reset-chain`)

A prior `genesis_pin.json` / `head_snap.json` in the same folder is still read once
and migrated. See [docs/ethean-redb-ssz-data-dir-2026-09-20.md](../storage/ethean-redb-ssz-data-dir-2026-09-20.md)
and [docs/ethean-data-dir-run-logs-2026-09-20.md](../pq-devnet/ethean-data-dir-run-logs-2026-09-20.md).
Console colors: [docs/ethean-console-log-colors-2026-09-20.md](../observability/ethean-console-log-colors-2026-09-20.md).
`--reset-chain` empties the folder first: [docs/reset-chain-wipes-data-dir-2026-09-20.md](../storage/reset-chain-wipes-data-dir-2026-09-20.md).

`--ephemeral` wins over `--data-dir` if both are set (logs a warning).

Console log level defaults to **INFO** (libp2p heartbeats stay quiet). Pass `-v`
for Ethean DEBUG, `-vv` for libp2p DEBUG, or set `RUST_LOG`. See
[docs/ethean-log-verbosity-2026-09-20.md](../observability/ethean-log-verbosity-2026-09-20.md).

On start, Ethean prints an ASCII identity banner and a start snapshot (network,
slots, roles, metrics) before the normal log dump — see
[docs/ethean-startup-banner-2026-09-20.md](../observability/ethean-startup-banner-2026-09-20.md)
(`--no-banner` to skip).

#### What the peer model (fixed genesis package) gives you

Same idea as Ream / Zeam / ethlambda / qlean-mini / Lantern / gean / Peam on
pq-devnets (lean-quickstart / `setup-genesis.sh`):

- **Resume after restart** — same `GENESIS_TIME` + data directory → continue
  from the last head (e.g. slot 48), not a new chain.
- **Shared chain identity** — multiple nodes (or a mixed-client mesh) load the
  **same** generated bundle.
- **Real pq-devnet ops** — generate once, reuse; matches lean-quickstart style.
- **Cleaner debugging** — finality / P2P bugs are not confused with accidental
  re-genesis.

#### What you trade away vs ephemeral smoke

- First start needs a durable path (`--data-dir`); files appear under that folder.
- Quick “does finality move?” smoke is slightly less one-liner friendly unless
  you pass `--ephemeral`.
- Regenerating genesis (`--reset-chain`, which empties `--data-dir`) starts a **new**
  chain — same as peers when they re-run `--generateGenesis`.

#### When to use which

| Goal | Prefer |
| --- | --- |
| Test / mesh like peer clients | Fixed package (`--data-dir`) |
| ~30s local finality smoke | Ephemeral (`--ephemeral`) |
| Production-like Lean client direction | Fixed package (`--data-dir`) |

Related: [docs/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](../peer-clients/peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md),
[docs/dual-mode-persist-and-ephemeral-2026-09-20.md](../storage/dual-mode-persist-and-ephemeral-2026-09-20.md).

### Local private mesh (no public bootnodes)

Same pattern as Ream/ethlambda: create a private 2-peer mesh for this run, write
peer A’s dialable address to `target/local-pq-mesh/nodes.multiaddrs`, then dial it.

```bash
./scripts/local-pq-mesh.sh
```

Details: [docs/local-pq-mesh-private-dial-2026-09-20.md](../pq-devnet/local-pq-mesh-private-dial-2026-09-20.md).

### Join an operator mesh (pq-devnet-4)

```bash
# Paste operator QUIC multiaddrs into the file, or pass them on the CLI:
ethean start --until-signal --network pq-devnet-4 \
  --bootnodes '/ip4/…/udp/…/quic-v1/p2p/…'

# Match operator gossip digest when they publish one (8 hex chars):
ethean start --until-signal --network pq-devnet-4 \
  --fork-digest aabbccdd --bootnodes '…'

# Or edit config/networks/pq-devnet-4.bootnodes and:
ethean start --until-signal --network pq-devnet-4
```

### Ready path: pq-devnet-5 (keep prepared)

Same binary and sync/crypto stack. Use when an operator D5 run publishes bootnodes:

```bash
# Explicit D5 label (not the CLI default while D5 is unreachable)
ethean start --until-signal --network pq-devnet-5

ethean start --until-signal --network pq-devnet-5 \
  --bootnodes '/ip4/…/udp/…/quic-v1/p2p/…' \
  --fork-digest aabbccdd

# File-based (preferred for long runs):
#   edit config/networks/pq-devnet-5.bootnodes
#   edit config/networks/pq-devnet-5.forkdigest
ethean start --until-signal --network pq-devnet-5

# Helpers
./scripts/run-pq-devnet-5.sh
```

### What you will see

- Terminal **tracing** logs: network label, bootnode dials (or offline warning), crypto gates, `/lean/v1/health` smoke, duty ticks.
- There is **no** `ethean monitor` subcommand and no Grafana UI in this binary yet.
- Health surface used internally: Lean `/lean/v1/…` (not Beacon `/eth/v1`).

More detail: [docs/deployment.md](../deployment.md), [docs/pq-devnet-operator-plug-in-checklist-2026-09-20.md](../pq-devnet/pq-devnet-operator-plug-in-checklist-2026-09-20.md), [docs/ethean-path-command-after-build-2026-09-20.md](../pq-devnet/ethean-path-command-after-build-2026-09-20.md), [docs/build-path-shim-quiet-success-2026-09-20.md](../process/build-path-shim-quiet-success-2026-09-20.md), [docs/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](../pq-devnet/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md), [docs/ream-empty-lean-peers-bootnodes-2026-09-20.md](../peer-clients/ream-empty-lean-peers-bootnodes-2026-09-20.md), [docs/working-client-pq-devnet-5-plan-2026-09-19.md](../pq-devnet/working-client-pq-devnet-5-plan-2026-09-19.md), [docs/pq-devnet-5-research-refresh-2026-09-19.md](../pq-devnet/pq-devnet-5-research-refresh-2026-09-19.md), [docs/blocks-by-range-quic-stream-2026-09-19.md](../networking/blocks-by-range-quic-stream-2026-09-19.md), [docs/blocks-by-range-serve-gap-warn-2026-09-20.md](../networking/blocks-by-range-serve-gap-warn-2026-09-20.md), [docs/serve-cache-seed-from-data-dir-2026-09-20.md](../networking/serve-cache-seed-from-data-dir-2026-09-20.md), [docs/persist-applied-blocks-durable-2026-09-20.md](../storage/persist-applied-blocks-durable-2026-09-20.md), [docs/durable-block-prune-finalized-keep-2026-09-20.md](../storage/durable-block-prune-finalized-keep-2026-09-20.md), [docs/durable-persist-prune-metrics-2026-09-20.md](../observability/durable-persist-prune-metrics-2026-09-20.md), [docs/grafana-durable-persist-prune-panels-2026-09-20.md](../observability/grafana-durable-persist-prune-panels-2026-09-20.md), [docs/prune-keep-slots-config-2026-09-20.md](../storage/prune-keep-slots-config-2026-09-20.md), [docs/durable-prune-stall-alert-2026-09-20.md](../observability/durable-prune-stall-alert-2026-09-20.md), [docs/range-serve-seed-metrics-2026-09-20.md](../observability/range-serve-seed-metrics-2026-09-20.md), [docs/grafana-range-serve-seed-panels-2026-09-20.md](../observability/grafana-range-serve-seed-panels-2026-09-20.md), [docs/c3-fork-digest-mesh-isolation-b1-refuse-2026-09-20.md](../networking/c3-fork-digest-mesh-isolation-b1-refuse-2026-09-20.md), [docs/b2-leanvm-refuse-d2-aggregator-subnets-2026-09-20.md](../lean-crypto/b2-leanvm-refuse-d2-aggregator-subnets-2026-09-20.md), [docs/d2-type1-aggregation-gossip-publish-2026-09-20.md](../networking/d2-type1-aggregation-gossip-publish-2026-09-20.md), [docs/d3-type2-from-type1-cache-2026-09-20.md](../lean-crypto/d3-type2-from-type1-cache-2026-09-20.md), [docs/b2-leanvm-ipc-frames-b4-aggpin-2026-09-20.md](../lean-crypto/b2-leanvm-ipc-frames-b4-aggpin-2026-09-20.md), [docs/b2-leanvm-ipc-spawn-exchange-2026-09-20.md](../lean-crypto/b2-leanvm-ipc-spawn-exchange-2026-09-20.md), [docs/b2-leanvm-mock-ipc-roundtrip-2026-09-20.md](../lean-crypto/b2-leanvm-mock-ipc-roundtrip-2026-09-20.md), [docs/b1-leansig-vendor-backend-compile-2026-09-20.md](../lean-crypto/b1-leansig-vendor-backend-compile-2026-09-20.md), [docs/b2-leanvm-ipc-live-probe-2026-09-20.md](../lean-crypto/b2-leanvm-ipc-live-probe-2026-09-20.md), [docs/attestation-committee-count-profile-subnets-2026-09-20.md](../networking/attestation-committee-count-profile-subnets-2026-09-20.md), [docs/hive-upstream-clients-ethean-dropin-2026-09-20.md](../hive-testing/hive-upstream-clients-ethean-dropin-2026-09-20.md), [docs/hive-leansig-cargo-features-2026-09-20.md](../hive-testing/hive-leansig-cargo-features-2026-09-20.md), [docs/attest-before-prove-empty-type1-2026-09-20.md](../lean-crypto/attest-before-prove-empty-type1-2026-09-20.md), [docs/b2-leanvm-ipc-split-ops-2026-09-20.md](../lean-crypto/b2-leanvm-ipc-split-ops-2026-09-20.md).

### Validator stub

```bash
ethean validator
```

Reports the crypto gate status. XMSS signing and verification are native
(`leansig=true` always); aggregate proofs stay fail-closed (`leanvm=false`) until a
leanVM prover is linked or reachable over IPC. Registry keys from
`--validator-registry` are decoded, checked against their `pubkey_hex`, and
installed on the native backend (see
[`docs/native-xmss-backend-2026-09-22.md`](../lean-crypto/native-xmss-backend-2026-09-22.md)).

## Monitoring and metrics

Long-run monitoring: head / justified / finalized / current slot, plus validators,
roles, readiness, peers, and lag gauges on provisioned Grafana boards.

**Two layers (easy to confuse):**

| Layer | What | When |
| --- | --- | --- |
| Scrape HTTP (`:9100`) | Ethean exports `/metrics` `/healthz` `/readyz` | **On by default** (disable with `--no-metrics`) |
| Prometheus UI + Grafana | Docker Compose in `deploy/observability` | Pass **`--observability-stack`** (or run `scripts/run-observability.*`) |

`ethean start --until-signal` alone does **not** open http://localhost:3000 or :9090.
Those ports need Docker. Your process metrics at http://127.0.0.1:9100/metrics already work.

### Process health APIs (live from the binary)

Metrics HTTP binds to `127.0.0.1:9100` unless overridden:

| URL | Expect | Meaning |
| --- | --- | --- |
| [http://127.0.0.1:9100/healthz](http://127.0.0.1:9100/healthz) | `200` + `ok` | Process alive |
| [http://127.0.0.1:9100/readyz](http://127.0.0.1:9100/readyz) | `200` + `ready` (or `503` while booting) | Storage/crypto/signer/network gates |
| [http://127.0.0.1:9100/metrics](http://127.0.0.1:9100/metrics) | Prometheus text | Slot gauges for Grafana |

```bash
curl -s http://127.0.0.1:9100/healthz
curl -s http://127.0.0.1:9100/readyz
curl -s http://127.0.0.1:9100/metrics | findstr ethean_head_slot
# Unix: curl -s http://127.0.0.1:9100/metrics | grep ethean_head_slot
```

Useful gauges: `ethean_head_slot`, `ethean_justified_slot`, `ethean_finalized_slot`,
`ethean_slot_current`, `ethean_peer_count`, `ethean_bootnode_count`,
`ethean_validator_count`, `ethean_finality_lag_slots`, `ethean_ready`,
`ethean_aggregator_enabled`.

If Grafana shows **1970** dates or **No data** on validators/roles, rebuild the
binary and reload dashboards:
[docs/grafana-no-data-and-1970-fix-2026-09-20.md](../observability/grafana-no-data-and-1970-fix-2026-09-20.md).

### Start with Grafana + Prometheus

```bash
# Starts docker compose (Grafana :3000, Prometheus :9090) then the node
ethean start --until-signal --network pq-devnet-4 --observability-stack

# Same for the D5 ready-path label
ethean start --until-signal --network pq-devnet-5 --observability-stack
```

| Flag | Default | Role |
| --- | --- | --- |
| `--metrics` | off | Keep scrape HTTP on (already the default; Hive/Ream alias) |
| `--observability-stack` | off | `docker compose up -d` for Prometheus + Grafana |
| `--no-metrics` | off | Disable scrape HTTP on :9100 |
| `--metrics-address` | `127.0.0.1` | Bind host for scrape (`ETHEAN_METRICS_ADDRESS`) |
| `--metrics-port` | `9100` | Bind port (must match Prometheus scrape) |
| `--until-signal` | off | Long-run until Ctrl-C |
| `--network` | `pq-devnet-4` | Label, or a path to Lean `config.yaml` (loads under `local`) |
| `--lean-config` | unset | Explicit `config.yaml` (wins over `--network` path) |
| `--bootnodes` | unset | `none`, CSV of QUIC multiaddrs / `enr:`, or `nodes.yaml` path |
| `--node-key` | unset | secp256k1 hex file (alias `--private-key-path`); else `<data-dir>/node.key` |
| `--socket-address` | `0.0.0.0` | QUIC listen interface |
| `--listen-port` | `9000` | UDP/QUIC port (alias `--socket-port`; `0` = ephemeral) |
| `--data-dir` | unset | Durable fixed genesis + head resume + `node.key` |
| `--ephemeral` | off | Force recent-genesis smoke (ignore `--data-dir`) |
| `--reset-chain` | off | Empty `--data-dir` (chain + logs) before start |
| `-v` / `--verbose` | off | More logs (`-v` DEBUG, `-vv` +libp2p, `-vvv` TRACE) |
| `--log-level` | unset | Max level (`info`/`debug`/`trace`; `RUST_LOG` wins) |
| `--no-banner` | off | Skip ASCII logo + start snapshot card |
| `--validators` | `4` | Local registry size (solo smoke) |
| `--is-aggregator` | off | Enable aggregator when a genesis file is given (mesh default off) |
| `--no-aggregator` | off | Disable aggregator (solo default **on**) |
| `--no-local-finality` | off | Disable solo head/finality advance (solo default **on**; mesh always off) |
| `--aggregate-subnet-ids` | unset | CSV of subnet ids (recorded; all subnets already subscribed) |
| `--attestation-committee-count` | unset | Override `ATTESTATION_COMMITTEE_COUNT` from `config.yaml` |
| `--checkpoint-sync-url` | unset | Accepted; empty-datadir SSZ fetch not yet applied |
| `--http-address` | `127.0.0.1` | Lean HTTP bind (`ETHEAN_HTTP_ADDRESS`) |
| `--http-port` | `5052` | Lean HTTP port |
| `--validator-registry` | unset | Alias `--validator-registry-path` |

Manual stack only (if you prefer not to use `--observability-stack`):

```bash
./scripts/run-observability.sh
# or: cd deploy/observability && docker compose up -d
```

| Service | Link |
| --- | --- |
| Grafana (anonymous viewer) | [http://localhost:3000](http://localhost:3000) |
| Dashboard | **Ethean Lean Clients Dashboard** + **Ethean Node Health** (folder Ethean) |
| Prometheus UI | [http://localhost:9090](http://localhost:9090) |
| Prometheus targets | [http://localhost:9090/targets](http://localhost:9090/targets) (`ethean` / `ethean-localhost` → UP) |
| Node scrape | [http://127.0.0.1:9100/metrics](http://127.0.0.1:9100/metrics) |

Healthy long-run: `ethean_slot_current` and `ethean_head_slot` climb; justified /
finalized follow when a mesh + aggregator exists. Flat finalized while head climbs
= finality stall (same failure mode Shariq caught on a 5-day Ream run).

Details: [docs/long-run-metrics-grafana-2026-09-20.md](../observability/long-run-metrics-grafana-2026-09-20.md),
[docs/ethean-grafana-richer-monitors-2026-09-20.md](../observability/ethean-grafana-richer-monitors-2026-09-20.md),
[docs/metrics-flag-starts-grafana-prometheus-2026-09-20.md](../observability/metrics-flag-starts-grafana-prometheus-2026-09-20.md),
[deploy/observability/README.md](../../deploy/observability/README.md).

There is no `ethean monitor` CLI. Do not use Beacon `/eth/v1/node/health` paths.

##  Configuration

### Basic Configuration (`config.toml`)

```toml
[network]
listen_address = "0.0.0.0:9000"
discovery_address = "0.0.0.0:9001"
max_peers = 100
target_peers = 50

[database]
path = "./data"
cache_size_mb = 512
enable_compression = true

[api]
enabled = true
address = "127.0.0.1:5052"
cors_origins = ["*"]

[logging]
level = "info"
format = "json"
file = "./logs/Ethean.log"

[metrics]
enabled = true
# Scrape HTTP (Prometheus). Default bind matches ethean CLI:
#   --metrics-address 127.0.0.1 --metrics-port 9100
address = "127.0.0.1"
port = 9100
```

### Advanced Configuration

```toml
[consensus]
proposer_boost = true
fork_choice_before_proposal = true
prepare_payload_lookahead = 4000

[validator]
graffiti = "Ethean Validator"
fee_recipient = "0x..."
builder_proposals = true

[database.backup]
enabled = true
interval_hours = 6
max_backups = 24
compression = true

[network.bandwidth]
max_upload_mbps = 100
max_download_mbps = 500
rate_limiting = true
```

## 🗄️ Database Management

### Backup & Recovery

```bash
# Create full backup
Ethean database backup --type full --output ./backups/

# Create incremental backup
Ethean database backup --type incremental --base ./backups/full_backup_123456

# Restore from backup
Ethean database restore --backup ./backups/full_backup_123456

# List available backups
Ethean database list-backups
```

### Database Operations

```bash
# Compact database
Ethean database compact

# Verify database integrity
Ethean database verify

# Export state
Ethean database export --state head --output state.json

# Import genesis state
Ethean database import --genesis genesis.ssz
```

### Cache Management

```bash
# Clear cache
Ethean cache clear

# Cache statistics
Ethean cache stats

# Optimize cache
Ethean cache optimize --target-size 1GB
```

##  Debugging & Troubleshooting

### Log Analysis

```bash
# View recent logs
tail -f ./logs/Ethean.log

# Filter error logs
grep "ERROR" ./logs/Ethean.log

# Analyze performance logs
Ethean logs analyze --performance --last 1h
```

### Debug Mode

```bash
# Start in debug mode
RUST_LOG=debug Ethean start

# Enable specific module debugging
RUST_LOG=Ethean::consensus=debug,Ethean::network=info Ethean start

# Debug with backtrace
RUST_BACKTRACE=1 Ethean start
```

### Common issues

- Blank Grafana/Prometheus (:3000 / :9090): the Docker daemon is not running. Scrape
  at :9100 still works without Docker.
- Empty bootnodes file: offline under the network label (expected). Paste operator
  multiaddrs or run scripts/local-pq-mesh.sh for a private mesh.
- ethean not found: ensure ~/.cargo/bin is on PATH after cargo build -p ethean.

See also [deployment.md](../deployment.md) and
[default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](../pq-devnet/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md).
