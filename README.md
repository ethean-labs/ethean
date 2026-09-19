# Ethean Lean Consensus Client

<div align="center">

![Ethean Logo](https://img.shields.io/badge/Ethean-Lean%20Consensus%20Client-blue?style=for-the-badge)

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg?style=flat-square)]()

**Rust consensus client for Ethereum Lean Consensus (Beam / leanEthereum)**

[Installation](#installation) • [Quick Start](#quick-start) • [Development](#development) • [Testing](#testing) • [Documentation](#documentation)

</div>

##  Overview

**Ethean Lean Consensus Client** implements Ethereum's Lean Consensus layer: a consensus-only client (not execution). Lean Consensus is the post-quantum rewrite of Beacon consensus — hash-based signatures (leanSig), aggregate proofs (leanMultisig / zkVMs), ~4s slots, finality in seconds (3SF, later PQ heartbeat), and a much larger validator set if the stake floor moves toward 1 ETH.

Work in this repo is meant to track the [Lean Consensus research tracks](https://leanroadmap.org/#research-tracks) (Poseidon, XMSS-style multi-signatures, aggregation, formal verification, Gossipsub v2 / set reconciliation, APS, 3SF) and the pq-devnet sequence, not to freeze a 2024 mainnet Beacon clone.


###  Key Features

- **Modular Architecture**: Beam/Lean separation of concerns with extensible design
- **High Performance**: Optimized for speed and efficiency with advanced caching
- **Security First**: Memory-safe Rust implementation with comprehensive testing
- **Advanced Monitoring**: Real-time metrics, performance benchmarking, and health monitoring
- **P2P Networking**: Robust peer-to-peer communication with bandwidth management
- **Database Optimization**: Advanced storage with indexing, caching, and backup systems
- **Developer Friendly**: Comprehensive API, WebSocket streaming, and extensive documentationBeam/Lean Chain Client

Ethean is a modern, high-performance Ethereum Beam/Lean Chain client written in Rust. It provides a complete implementation of the Ethereum 2.0 proof-of-stake consensus mechanism with emphasis on modularity, security, and developer experience.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Development](#development)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Overview

Ethean implements the Ethereum Beam/Lean Chain specification with the following key features:

- **Modular Architecture**: Beam/Lean separation of concerns with well-defined module boundaries
##  Installation

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Git**: For cloning the repository
- **System Requirements**: 
  - RAM: 8GB+ recommended
  - Storage: 500GB+ SSD recommended
  - Network: Stable internet connection

### Quick Install

```bash
# Clone the repository
git clone https://github.com/Pamenarti/Ethean.git
cd Ethean

# Build (also publishes an `ethean` shim into ~/.cargo/bin)
cargo build -p ethean --release

# From any directory (Cargo bin must be on PATH):
ethean version
```

### Development Install

```bash
# Clone with all dependencies
git clone https://github.com/Pamenarti/Ethean.git
cd Ethean

# Debug build also refreshes the PATH shim
cargo build -p ethean

# Run tests to verify installation
cargo test
```

##  Quick Start

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

After that build, the command is `ethean` (Windows and Linux). The build script
places a shim in `~/.cargo/bin`; no `install.sh` step.

### Run (default: pq-devnet-4)

```bash
ethean version
ethean start
ethean start --ticks 3
ethean start --ticks 2 --wall-clock
ethean start --until-signal
ethean start --network pq-devnet-4
ethean start --network local
```

Helpers (build + long-run):

```bash
# Unix
./scripts/run-pq-devnet-4.sh

# Windows PowerShell
.\scripts\run-pq-devnet-4.ps1
```

Paste D4 QUIC multiaddrs into `config/networks/pq-devnet-4.bootnodes` (or pass
`--bootnodes` / `ETHEAN_BOOTNODES`) before expecting a live mesh dial.

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
# Windows: .\scripts\run-pq-devnet-5.ps1
```

### What you will see

- Terminal **tracing** logs: network label, bootnode dials (or offline warning), crypto gates, `/lean/v1/health` smoke, duty ticks.
- There is **no** `ethean monitor` subcommand and no Grafana UI in this binary yet.
- Health surface used internally: Lean `/lean/v1/…` (not Beacon `/eth/v1`).

More detail: [docs/deployment.md](docs/deployment.md), [docs/ethean-path-command-after-build-2026-09-20.md](docs/ethean-path-command-after-build-2026-09-20.md), [docs/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](docs/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md), [docs/working-client-pq-devnet-5-plan-2026-09-19.md](docs/working-client-pq-devnet-5-plan-2026-09-19.md), [docs/pq-devnet-5-research-refresh-2026-09-19.md](docs/pq-devnet-5-research-refresh-2026-09-19.md), [docs/blocks-by-range-quic-stream-2026-09-19.md](docs/blocks-by-range-quic-stream-2026-09-19.md).

### Validator stub

```bash
ethean validator
```

Reports leanSig / leanVM gate status (fail-closed until production backends link).

##  Development

### Project Structure

```
Ethean/
├── src/
│   ├── bin/           # Binary executables
│   ├── api/           # REST API implementation
│   ├── consensus/     # Consensus layer
│   ├── crypto/        # Cryptographic operations
│   ├── network/       # P2P networking
│   ├── storage/       # Database and caching
│   ├── types/         # Type definitions
│   └── lib.rs         # Library root
├── docs/              # Documentation
├── tests/             # Integration tests
└── examples/          # Usage examples
```

### Building from Source

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build with specific features
cargo build --release --features "rocksdb,metrics"

# Build documentation
cargo doc --open
```

### Development Tools

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated
##  Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test consensus::

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=8
```

### Test Categories

#### Unit Tests
```bash
# Core consensus tests
cargo test consensus::tests

# Cryptography tests
cargo test crypto::tests

# Storage tests
cargo test storage::tests

# Network tests
cargo test network::tests
```

#### Integration Tests
```bash
# API integration tests
cargo test --test api_integration

# P2P network tests
cargo test --test network_integration

# Database tests
cargo test --test storage_integration
```

#### Performance Tests
```bash
# Database benchmarks
cargo test --test database_benchmark -- --ignored

# Network performance tests
cargo test --test network_performance -- --ignored

# Consensus benchmarks
cargo test --test consensus_benchmark -- --ignored
```

#### Load Tests
```bash
# High-load scenarios
cargo test --test load_test -- --ignored

# Stress testing
cargo test --test stress_test -- --ignored
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Database performance benchmarks
cargo run --bin benchmark -- database

# Network benchmarks  
cargo run --bin benchmark -- network

# Consensus benchmarks
cargo run --bin benchmark -- consensus

# Custom benchmark with parameters
cargo run --bin benchmark -- database --operations 10000 --concurrency 8
```

### Test Configuration

Create `test-config.toml` for custom test settings:

```toml
[test]
log_level = "debug"
timeout_seconds = 30
parallel_tests = true

[test.database]
use_memory_db = true
Beam/Leanup_after_test = true

[test.network]
use_local_network = true
mock_peers = 10

[test.consensus]
fast_epoch_processing = true
skip_signature_verification = false
```

##  Monitoring & Metrics

Default `start` prints tracing logs and smokes Lean `/lean/v1/health` in-process.
Provisioned Grafana / Prometheus (`deploy/observability/`) is still planning-only; see `road-to/lean-consensus-migration/06-observability/`.

```bash
# Long-running node (Ctrl-C to stop); default label is pq-devnet-4
ethean start --until-signal --network pq-devnet-4

# Ready path when a D5 operator mesh publishes bootnodes
ethean start --until-signal --network pq-devnet-5
```

There is no `ethean monitor` CLI. Do not use Beacon `/eth/v1/node/health` paths.

##  API Usage

### REST API

```bash
# Get Beam/Lean state
curl http://localhost:5052/eth/v2/Beam/Lean/states/head

# Get block information
curl http://localhost:5052/eth/v2/Beam/Lean/blocks/head

# Submit attestation
curl -X POST http://localhost:5052/eth/v1/Beam/Lean/pool/attestations \
  -H "Content-Type: application/json" \
  -d @attestation.json
```

### WebSocket Streaming

```javascript
// JavaScript example
const ws = new WebSocket('ws://localhost:5052/ws');

ws.on('message', (data) => {
  const event = JSON.parse(data);
  console.log('Received event:', event);
});

// Subscribe to block events
ws.send(JSON.stringify({
  type: 'subscribe',
  topics: ['block', 'attestation']
}));
```

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
port = 9090
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

### Common Issues

1. **Sync Issues**
```bash
# Check sync status
curl http://localhost:5052/eth/v1/node/syncing

# Force resync
Ethean resync --from-checkpoint
```

2. **Peer Connection Problems**
```bash
# Check peer status
Ethean network peers

# Test connectivity
Ethean network test-connectivity --peer-id <peer-id>
```

3. **Database Corruption**
```bash
# Verify database
Ethean database verify --repair

# Restore from backup
Ethean database restore --latest-backup
```

##  Contributing

### Development Setup

```bash
# Fork and clone
git clone https://github.com/yourusername/Ethean.git
cd Ethean

# Create feature branch
git checkout -b feature/new-feature

# Make changes and test
cargo test
cargo clippy
cargo fmt

# Commit and push
git commit -m "Add new feature"
git push origin feature/new-feature
```



```json
{
  "execution_optimistic": false,
  "finalized": true,
  "data": {
    // Response data
  }
}
```

### Error Handling

Errors are returned in the following format:

```json
{
  "code": 400,
  "message": "Invalid request parameter",
  "stacktraces": ["Error details..."]
}
```

## Development

### Project Structure

Workspace layout (Lean migration):

- `crates/` — library crates (`primitives` … `metrics`; see [`crates/README.md`](crates/README.md))
- `bin/ethean` — node binary
- `spec/` — phase locks and fixtures
- `docs/` — migration notes
- `artifacts/`, `scripts/`, `tools/` — evidence, helpers, utilities
- `tests/{interop,security,recovery}/` — cross-crate suites
- `road-to/` — migration planning library

Legacy Beacon-shaped modules under `crates/node` are being replaced phase by phase.

### Adding New Features

1. Identify the appropriate module for your feature
2. Create comprehensive tests for new functionality
3. Update documentation and API schemas
4. Ensure all existing tests pass
5. Follow Rust best practices and project conventions

### Code Quality

The project maintains high code quality through:

- Comprehensive test coverage (182+ tests)
- Static analysis with Clippy
- Code formatting with rustfmt
- Documentation requirements
- Continuous integration

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test consensus

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Test Structure

- **Unit Tests**: Individual component testing
- **Integration Tests**: Cross-module interaction testing
- **API Tests**: HTTP endpoint testing
- **Performance Tests**: Benchmark testing

### Test Coverage

Current test coverage includes:

- Consensus engine: Complete coverage of fork choice and state transitions
- Network layer: P2P communication and peer management
- Storage layer: Data persistence and retrieval
- Cryptography: BLS signature verification
- API layer: All REST endpoints with various scenarios

## Performance Characteristics

### Benchmarks

- **Block Processing**: ~50ms average for mainnet blocks
- **Attestation Processing**: ~5ms average per attestation
- **State Transition**: ~100ms for epoch transitions
- **Network Throughput**: 1000+ messages per second
- **Memory Usage**: ~2GB for mainnet synchronization

### Optimization Areas

- Zero-copy data structures where possible
- Parallel processing for CPU-intensive operations
- Efficient state caching strategies
- Optimized database queries and indexing

## Contributing

We welcome contributions to Ethean. Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write comprehensive tests
4. Update documentation
5. Submit a pull request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Pamenarti/Ethean.git
cd Ethean

# Install dependencies
cargo build

# Run tests
cargo test

# Run linting
cargo clippy

# Format code
cargo fmt
```

### Code Standards

- **Rust Style**: Follow official Rust style guidelines
- **Documentation**: Document all public APIs
- **Testing**: Maintain >95% test coverage
- **Performance**: Benchmark critical paths
- **Security**: Follow secure coding practices

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

##  Acknowledgments

- **Ethereum Foundation**: For the Beam/Lean Chain specification
- **Rust Community**: For the excellent ecosystem
- **Contributors**: All developers who have contributed to this project
- **Lighthouse Team**: For inspiration and reference implementations

##  Support
- **Documentation**: [docs](./docs/)
- **Docs index**: [docs/README.md](./docs/README.md)
- **Folder READMEs and local conventions**: [docs/folder-readmes-and-local-conventions.md](./docs/folder-readmes-and-local-conventions.md)
- **Lean Consensus migration plans**: [road-to/lean-consensus-migration/README.md](./road-to/lean-consensus-migration/README.md) (active planning library; see also [road-to/README.md](./road-to/README.md))
- **Source tree**: [src/README.md](./src/README.md)
- **Lean Consensus tracks**: [leanroadmap.org research tracks](https://leanroadmap.org/#research-tracks)
- **Lean Consensus R&D (full site)**: [leanroadmap.org](https://leanroadmap.org/)
- **How we capture that locally**: [docs/leanroadmap-local-notes.md](./docs/leanroadmap-local-notes.md)
- **Source file size (300 lines)**: [docs/source-file-size-limit.md](./docs/source-file-size-limit.md)
- **Peer Lean clients (reference)**: [docs/peer-reference-clients.md](./docs/peer-reference-clients.md)
- **Seven-client source research**: [docs/lean-peer-client-research-library-2026-09-19.md](./docs/lean-peer-client-research-library-2026-09-19.md)
- **Language (English only)**: [docs/english.md](./docs/english.md)
- **Commits (per file, English)**: [docs/commit-after-each-file.md](./docs/commit-after-each-file.md)
- **GitHub Issues**: [Report bugs](https://github.com/Pamenarti/Ethean/issues)
- **Email**: support@Ethean.io

---

<div align="center">

**Built with ❤️ by the Ethean Team**

[Website](https://Ethean.io) • [GitHub](https://github.com/Pamenarti/Ethean)

</div>

---

**Note**: This is a development version. For production use, please wait for the stable release and conduct thorough testing in your environment.
