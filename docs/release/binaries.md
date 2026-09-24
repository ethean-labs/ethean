# Pre-built ethean binaries

GitHub Releases can attach platform archives built by
[`.github/workflows/release-binaries.yml`](../../.github/workflows/release-binaries.yml).

## Targets

| System | Architecture | Rust target | Archive |
| --- | --- | --- | --- |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | `.zip` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux | aarch64 | `aarch64-unknown-linux-gnu` (via `cross`) | `.tar.gz` |
| macOS | aarch64 | `aarch64-apple-darwin` | `.tar.gz` |
| macOS | x86_64 | `x86_64-apple-darwin` | `.tar.gz` |

Naming: `ethean-v<VERSION>-<target>.{tar.gz|zip}` plus a sibling
`ethean-v<VERSION>-<target>.sha256`. Each archive holds `ethean` and
`ethean-prover`; keep them in the same directory so the node finds the prover
(or set `ETHEAN_PROVER_BIN`).

Windows builds use [`vendor/leanvm-windows/`](../../vendor/leanvm-windows/README.md)
overlays so leanVM `system-info` / `zk-alloc` compile without Unix `getrusage` /
sparse `mmap`. The proving arena stays disabled on Windows (System allocator).

## Features in release builds

- Packages: `ethean` and `ethean-prover` (`cargo build -p ethean -p ethean-prover --release --locked`)
- Default features: `libp2p-quic` enabled
- XMSS is native; aggregate proofs use leanMultisig at leanVM `e2592df4` (pq-devnet-4 pin)
- Optional `rocksdb` storage feature is **not** enabled (redb data-dir path)

## Local packaging

```powershell
cargo build -p ethean -p ethean-prover --release --locked --target x86_64-pc-windows-msvc
powershell -NoProfile -File tools/release/package-ethean.ps1 -Version 0.1.53 -Target x86_64-pc-windows-msvc
```

```bash
cargo build -p ethean -p ethean-prover --release --locked --target x86_64-unknown-linux-gnu
tools/release/package-ethean.sh 0.1.53 x86_64-unknown-linux-gnu
```

## Attach to an existing release

```bash
gh workflow run release-binaries.yml -f tag=v0.1.53
```

Or push a new `v*` tag. The publish job uploads archives with `--clobber` and
prepends / refreshes the `## Binaries` section in the release notes.

## Docker images

Multi-arch container images (`ethean` + `ethean-prover`) are published by
[`.github/workflows/docker-image.yml`](../../.github/workflows/docker-image.yml)
to `ghcr.io/<github-owner>/ethean`. Tags, local build, and the Hive smoke
command: [`docker-images.md`](./docker-images.md).
