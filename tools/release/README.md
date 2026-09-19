# Release tooling

Scripts and manifests for reproducible builds, SBOM stubs, and artifact manifests.

- `repro-build.ps1` — clean `cargo build --release --locked` and record hashes
- `sbom.ps1` — placeholder SBOM generation notes (cargo-cyclonedx / cargo-deny)
- `artifact-manifest.toml` — fields required in a release package
- `legacy-scan.ps1` — soft scan for retirement symbols; **hard-fail** if `blst` / `blstrs` / `bls12_381` appear under `crates/` or `bin/`
