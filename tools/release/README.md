# Release tooling

Scripts and manifests for reproducible builds, SBOM stubs, release archives and artifact manifests. Ethean targets Linux (and macOS for release archives); there is no Windows tooling.

- `package-ethean.sh` — archive `ethean` and `ethean-prover` plus a SHA-256 file for one rustc target
- `render-binaries-notes.sh` — `## Binaries` markdown for GitHub Release bodies
- `.github/workflows/release-binaries.yml` — Linux / macOS matrix build and release upload
- `repro-build.sh` — `cargo build --release --locked` for the core crates and record toolchain, git head, and `Cargo.lock` hash in `artifacts/phase-13/repro/build-meta.txt`
- `sbom.sh` — SBOM stub: records the `Cargo.lock` SHA-256 under `artifacts/phase-13/sbom/` until cargo-cyclonedx is wired in CI
- `artifact-manifest.toml` — fields required in a release package
- `legacy-scan.sh` — soft scan for retirement symbols (reports in `artifacts/phase-13/legacy-scan/`); **hard-fail** if `blst` / `blstrs` / `bls12_381` appear under `crates/` or `bin/`
- `fetch-leanspec-fixtures.sh` — download the leanSpec production fixture archive into `.cache/leanspec-fixtures/`, verify size / SHA-256 against `spec/fixtures/phase-00/manifest.toml`, and extract it

Operator doc: [`docs/release/binaries.md`](../../docs/release/binaries.md).
