# B1 leanSig production backend — local vendor compile path (2026-09-20)

## Status

`ethean-crypto` feature `leansig-backend` **compiles and unit-tests green** when leanSig
is served from a local num-bigint 0.5 vendor tree. The committed git dependency alone
still fails with dual `num_bigint` (0.4 vs 0.5) against Plonky3.

## What landed

- `tools/release/check-leansig-backend.ps1`
  - Ensures `bazalinacaklar/leanSig-patched` (runs vendor script if missing)
  - Writes gitignored `.cargo/config.toml` with
    `[patch."https://github.com/leanEthereum/leanSig"]`
  - `cargo check -p ethean-crypto --features leansig-backend` (optional `-Test`)
  - Restores `Cargo.lock` and deletes the local config
- `.cargo/config.toml.example` + `.gitignore` entries for local config / lock backup
- `LeanSigGate.vendor_bigint_patch_required` / `LEANSIG_VENDOR_BIGINT_PATCH_REQUIRED`
  (still `true` until upstream bumps bigint)
- Boot log field `leansig_vendor_patch`
- Feature-aware `ffi_status` tests; fast PROD key_gen input validation; ignored full
  XMSS roundtrip (`-- --ignored`, minutes-class for lifetime `2^32`)

## Operator recipe

```powershell
tools/release/vendor-leansig-bigint-fix.ps1
tools/release/check-leansig-backend.ps1 -Test
```

Do **not** commit `.cargo/config.toml`, path overrides in `Cargo.toml`, or a
path-patched `Cargo.lock`.

## Verified locally

- `cargo check -p ethean-crypto --features leansig-backend` (with `[patch]`)
- `cargo test -p ethean-crypto --features leansig-backend --lib` → 38 passed, 1 ignored
- Default (no feature) `ffi_status` still reports leanSig not ready

## Still open (B1 residual / next)

| Gap | Notes |
| --- | --- |
| Upstream leanSig `num-bigint` 0.5 | Required before git dep works without `[patch]` |
| Default CI `leansig-backend` | Off until upstream pin; local check script only |
| Full PROD keygen in CI | Ignored; too slow for lifetime `2^32` |
| leanVM `protocol_ready` | B2/B3 (spawn + real prover) |
