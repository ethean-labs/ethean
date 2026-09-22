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

Naming: `ethean-v<VERSION>-<target>.{zip|tar.gz}` plus a sibling
`ethean-v<VERSION>-<target>.sha256`.

## Features in release builds

- Package: `ethean` only (`cargo build -p ethean --release --locked`)
- Default features: `libp2p-quic` enabled
- `test-aggregate` **off** (no keyless Type-2 proofs in shipped binaries)
- Optional `rocksdb` storage feature is **not** enabled (redb data-dir path)

## Local packaging

```powershell
cargo build -p ethean --release --locked --target x86_64-pc-windows-msvc
powershell -NoProfile -File tools/release/package-ethean.ps1 -Version 0.1.47 -Target x86_64-pc-windows-msvc
```

```bash
cargo build -p ethean --release --locked --target x86_64-unknown-linux-gnu
tools/release/package-ethean.sh 0.1.47 x86_64-unknown-linux-gnu
```

## Attach to an existing release

After the workflow lands on `master`:

```bash
gh workflow run release-binaries.yml -f tag=v0.1.47
```

Or push a new `v*` tag. The publish job uploads archives with `--clobber` and
prepends / refreshes the `## Binaries` section in the release notes.
