# Installation

How to get a working Ethean toolchain and binary.

Short overview: [root README — Installation](../../README.md#installation).

### Prerequisites

- **Rust**: pin in [`rust-toolchain.toml`](../../rust-toolchain.toml) via [rustup](https://rustup.rs/)
- **Git**: For cloning the repository
- **System Requirements**: 
  - RAM: 8GB+ recommended
  - Storage: 500GB+ SSD recommended
  - Network: Stable internet connection

### Quick Install

```bash
# Clone the repository
git clone https://github.com/ethean-labs/ethean.git
cd Ethean

# Build (also publishes an `ethean` shim into ~/.cargo/bin)
cargo build -p ethean --release

# From any directory (Cargo bin must be on PATH):
ethean version
```

### Development Install

```bash
# Clone with all dependencies
git clone https://github.com/ethean-labs/ethean.git
cd Ethean

# Debug build also refreshes the PATH shim
cargo build -p ethean

# Run tests to verify installation
cargo test
```
