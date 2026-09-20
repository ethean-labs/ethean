# Type-2 block-root binding and leanSig pin checks (2026-09-19)

## Type-2 statement

- `type2_statement_for_block` components (ordered):
  1. body root
  2. attestation-data roots
  3. full block tree root (same root local proposer XMSS signs)
- Remotes reconstruct the statement from `SignedBlock.block` alone (no out-of-band sig required)
- Proposer wire signature remains off the Type-2 blob until leanMultisig merge params land

## leanSig gate / vendor

- `LeanSigGate` exposes `pinned_rev` (`LEANSIG_REV`) and `feature_enabled`
- `tools/release/check-leansig-vendor.ps1` now verifies:
  - patch file exists
  - `LEANSIG_REV` in `xmss/config.rs` matches expected rev
  - same rev in `crates/crypto/Cargo.toml`
  - optional local vendor tree has `num-bigint` 0.5
- Client boot logs leanSig feature + pin next to leanVM gate fields

## Verification

```text
cargo test -p ethean-transition --lib type2
cargo test -p ethean-crypto --lib ffi_status
cargo test -p ethean-node --lib
powershell -File tools/release/check-leansig-vendor.ps1
```

## Still open

- Upstream leanSig `num-bigint` 0.5 so git dep works without path override
- Set `LEANVM_FFI_LINKED` after real lean-multisig FFI symbols link
- Embed proposer XMSS bytes into the Type-2 proof blob when leanMultisig specifies the encoding
