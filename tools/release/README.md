# Release tooling

Scripts and manifests for reproducible builds, SBOM stubs, and artifact manifests.

- `package-ethean.sh` / `package-ethean.ps1` — archive + SHA256 for one rustc target
- `render-binaries-notes.sh` — `## Binaries` markdown for GitHub Release bodies
- `.github/workflows/release-binaries.yml` — Windows / Linux / macOS matrix upload
- `repro-build.ps1` — clean `cargo build --release --locked` and record hashes
- `sbom.ps1` — placeholder SBOM generation notes (cargo-cyclonedx / cargo-deny)
- `artifact-manifest.toml` — fields required in a release package
- `legacy-scan.ps1` — soft scan for retirement symbols; **hard-fail** if `blst` / `blstrs` / `bls12_381` appear under `crates/` or `bin/`
- `check-libclang.ps1` — locate `libclang.dll` for `ethean-storage/rocksdb` feature builds on Windows

Operator doc: [`docs/release/binaries.md`](../../docs/release/binaries.md).
