# Pre-built ethean binaries

GitHub Releases can attach platform archives built by
[`.github/workflows/release-binaries.yml`](../../.github/workflows/release-binaries.yml).

## Targets

| System | Architecture | Rust target | Archive |
| --- | --- | --- | --- |
| Windows (experimental) | x86_64 | `x86_64-pc-windows-msvc` | `.zip` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` (via `cross`) | `.tar.gz` |
| macOS | aarch64 | `aarch64-apple-darwin` | `.tar.gz` |
| macOS | x86_64 | `x86_64-apple-darwin` | `.tar.gz` |

Naming: `ethean-v<VERSION>-<target>.{tar.gz|zip}` plus a sibling
`ethean-v<VERSION>-<target>.sha256`. Each archive holds `ethean` and
`ethean-prover`; keep them in the same directory so the node finds the prover
(or set `ETHEAN_PROVER_BIN`).

Windows is experimental: leanMultisig at leanVM `e2592df4` uses Unix-only system
calls (`getrusage`, `mmap`) and does not compile for Windows yet, so the Windows
job is allowed to fail and its row appears only once an archive is produced.

## Features in release builds

- Packages: `ethean` and `ethean-prover` (`cargo build -p ethean -p ethean-prover --release --locked`)
- Default features: `libp2p-quic` enabled
- XMSS is native; aggregate proofs use leanMultisig at leanVM `e2592df4` (pq-devnet-4 pin)
- Optional `rocksdb` storage feature is **not** enabled (redb data-dir path)

## Local packaging

```bash
cargo build -p ethean -p ethean-prover --release --locked --target x86_64-unknown-linux-gnu
tools/release/package-ethean.sh 0.1.48 x86_64-unknown-linux-gnu
```

## Attach to an existing release

After the workflow lands on `master`:

```bash
gh workflow run release-binaries.yml -f tag=v0.1.47
```

Or push a new `v*` tag. The publish job uploads archives with `--clobber` and
prepends / refreshes the `## Binaries` section in the release notes.
