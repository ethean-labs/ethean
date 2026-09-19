# ethean-crypto

Lean Consensus XMSS wire surface and Type-1 / Type-2 aggregation for Ethean.

- PROD_CONFIG sizes: public key **52** bytes, signature **2536** bytes, dimension **46**, lifetime `2^32`.
- Aggregation: max proof **524288** bytes, `LOG_INV_RATE=2`, leanVM pin `e2592df4…`.
- Production backends fail closed unless `leansig-backend` / `leanvm-backend` compile.
- Default `test-hmac` / `test-aggregate` provide real (not always-true) verify for unit tests.

## leanSig backend (local vendor)

Pinned git rev `c08a3bae…` still declares `num-bigint 0.4` while Plonky3 wants `0.5`.
Until upstream unifies:

1. `pwsh tools/release/vendor-leansig-bigint-fix.ps1` (applies `vendor/leansig/num-bigint-0.5.patch`)
2. Temporarily point `crates/crypto/Cargo.toml` at `../../bazalinacaklar/leanSig-patched`
3. `cargo check -p ethean-crypto --features leansig-backend`
4. Restore the git dep before committing

## leanVM backend

`leanvm-backend` compiles `backend_leanvm` against pin `e2592df4…`.
`FfiStatus.leanvm` stays false until `LEANVM_FFI_LINKED` is flipped after real FFI/process link.
With `test-aggregate` (default), unit tests keep using the statement-bound synthetic verifier.
