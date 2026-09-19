# leanSig vendor overlay

Upstream leanSig (`c08a3bae…`) pins `num-bigint = "0.4.6"` while unpinned
Plonky3 resolves `num-bigint 0.5`, so `BigUint` types clash when enabling
`ethean-crypto/leansig-backend` against the git dependency alone.

## Commit-clean pieces

- [`num-bigint-0.5.patch`](./num-bigint-0.5.patch) — only dependency bump
- [`../../tools/release/vendor-leansig-bigint-fix.ps1`](../../tools/release/vendor-leansig-bigint-fix.ps1) — clones pin into
  `bazalinacaklar/leanSig-patched` (gitignored) and applies this patch

## Build

```powershell
pwsh tools/release/vendor-leansig-bigint-fix.ps1
# Temporarily in crates/crypto/Cargo.toml (do not commit):
# leansig = { path = "../../bazalinacaklar/leanSig-patched", package = "leansig", optional = true }
cargo check -p ethean-crypto --features leansig-backend
```

Do not vendor the full leanSig tree into git. Prefer upstream fixing the pin.
