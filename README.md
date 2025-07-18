# Panro - Ethereum Beacon Chain Client

Panro is a modern, high-performance Ethereum Beacon Chain client written in Rust. It provides a complete implementation of the Ethereum 2.0 proof-of-stake consensus mechanism with emphasis on modularity, security, and developer experience.

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

Panro implements the Ethereum Beacon Chain specification with the following key features:

- **Modular Architecture**: Clean separation of concerns with well-defined module boundaries
- **High Performance**: Rust-based implementation with async/await patterns for optimal performance
- **REST API**: Complete Ethereum Beacon API implementation with OpenAPI documentation
- **Production Ready**: Comprehensive error handling, logging, and monitoring capabilities
- **Developer Friendly**: Extensive documentation and testing coverage

### Key Components

- **Consensus Engine**: Full proof-of-stake consensus implementation
- **Network Layer**: libp2p-based P2P networking with gossip protocol
- **Storage Layer**: RocksDB-based persistent storage with state management
- **Cryptography**: BLS signature verification and cryptographic primitives
- **Validator Management**: Complete validator lifecycle management
- **REST API**: Ethereum Beacon API compliant HTTP interface

## Architecture

### Core Modules

```
src/
├── consensus/          # Consensus engine and fork choice
├── network/           # P2P networking and peer management
├── storage/           # Data persistence and state management
├── crypto/            # Cryptographic primitives and BLS signatures
├── types/             # Core data structures and primitives
├── api/               # REST API implementation
├── utils/             # Utility functions and helpers
└── lib.rs             # Main library entry point
```

### Data Flow

1. **Network Layer** receives blocks and attestations from peers
2. **Consensus Engine** validates and processes consensus messages
3. **Storage Layer** persists state and maintains chain history
4. **API Layer** provides HTTP interface for external clients
5. **Validator Management** handles validator duties and operations

## Installation

### Prerequisites

- Rust 1.70 or higher
- Git
- OpenSSL development libraries

### Build from Source

```bash
git clone https://github.com/Pamenarti/Panro.git
cd Panro
cargo build --release
```

### Run Tests

```bash
cargo test
```

## Usage

### Basic Usage

```bash
# Run with default configuration
./target/release/panro

# Run with custom configuration
./target/release/panro --config config.toml

# Run with specific network
./target/release/panro --network mainnet
```

### Configuration

Create a `config.toml` file:

```toml
[network]
listen_addr = "0.0.0.0:9000"
discovery_port = 9000
max_peers = 50

[api]
bind_addr = "127.0.0.1:5052"
enable_cors = true
max_request_size = 1048576

[consensus]
genesis_time = 1606824000
genesis_validators_root = "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"

[storage]
data_dir = "./data"
```

### API Usage

The REST API is available at `http://localhost:5052` by default.

#### Get Genesis Information

```bash
curl http://localhost:5052/eth/v1/beacon/genesis
```

#### Get Beacon State

```bash
curl http://localhost:5052/eth/v1/beacon/states/head/root
```

#### Get Validator Duties

```bash
curl http://localhost:5052/eth/v1/validator/duties/attester/12345
```

## API Documentation

### Beacon API Endpoints

#### Genesis and Configuration
- `GET /eth/v1/beacon/genesis` - Get genesis information
- `GET /eth/v1/config/fork_schedule` - Get fork schedule
- `GET /eth/v1/config/spec` - Get configuration specification

#### Beacon State
- `GET /eth/v1/beacon/states/{state_id}/root` - Get state root
- `GET /eth/v1/beacon/states/{state_id}/fork` - Get fork information

#### Validator Operations
- `GET /eth/v1/validator/duties/attester/{epoch}` - Get attester duties
- `GET /eth/v1/validator/duties/proposer/{epoch}` - Get proposer duties
- `POST /eth/v1/validator/blocks` - Submit block
- `POST /eth/v1/validator/attestations` - Submit attestations

#### Node Information
- `GET /eth/v1/node/identity` - Get node identity
- `GET /eth/v1/node/peers` - Get connected peers
- `GET /eth/v1/node/health` - Get node health status
- `GET /eth/v1/node/version` - Get node version

#### Debug Endpoints
- `GET /eth/v1/debug/beacon/states/{state_id}` - Get beacon state
- `GET /eth/v1/debug/beacon/heads` - Get beacon heads
- `GET /eth/v1/debug/fork_choice` - Get fork choice information

### Response Format

All API responses follow the standard format:

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

The project follows a modular architecture with clear separation of concerns:

- **consensus/**: Implements proof-of-stake consensus logic
- **network/**: Handles P2P networking and peer management
- **storage/**: Manages data persistence and state storage
- **crypto/**: Provides cryptographic primitives and BLS operations
- **api/**: Implements REST API with Ethereum Beacon API compliance
- **types/**: Defines core data structures and type definitions

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

We welcome contributions to Panro. Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write comprehensive tests
4. Update documentation
5. Submit a pull request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Pamenarti/Panro.git
cd Panro

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

- Follow Rust idioms and best practices
- Write comprehensive documentation
- Maintain test coverage above 90%
- Use meaningful variable and function names
- Follow the existing code style

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

- Ethereum Foundation for the Beacon Chain specification
- Rust community for excellent tooling and libraries
- Contributors and maintainers of dependent crates

## Contact

For questions, issues, or contributions, please visit our GitHub repository or contact the maintainers.

---

**Note**: This is a development version. For production use, please wait for the stable release and conduct thorough testing in your environment.
