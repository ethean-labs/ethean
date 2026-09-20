# Phase 13 — Security, performance, and release (2026-09-19)

## Summary

Phase 13 lands release **scaffolding** and runbooks; full fuzz/chaos/soak/SBOM signing remain open before targeted-devnet promotion.

### Tooling
- `rust-toolchain.toml` → **1.98.1** (authoring host)
- `tools/release/`: repro-build (ps1/sh), sbom stubs, artifact-manifest, legacy-scan
- `docs/release/`: gates, upgrade, rollback

### Test folders
`tests/{fuzz,chaos,soak,retirement,release}/` READMEs for campaign harnesses

### Pins
`spec/pins/phase-13.lock.toml` records promotion sequence and open gates (including pre-existing `ethean-node` compile debt)

## Non-waivable
Crypto isolation, XMSS leaf safety, checkpoint trust boundary, no Beacon `/eth/v1`, signer journal immutability on rollback

## Next for promotion
1. Clear `ethean-node` compile debt / purge blst from production paths  
2. Hard-fail legacy scan  
3. Run fuzz/chaos/soak with evidence under `artifacts/phase-13/`  
4. Two-builder repro verify + signed SBOM  
5. Targeted devnet
