# leanSig vendor overlay

Upstream leanSig (`c08a3bae…`) pins `num-bigint = "0.4.6"` while unpinned
Plonky3 resolves `num-bigint 0.5`, so `BigUint` types clash when enabling
`ethean-crypto/leansig-backend` against the git dependency alone.

## Commit-clean pieces

- [`num-bigint-0.5.patch`](./num-bigint-0.5.patch) — only dependency bump
- [`../../tools/release/vendor-leansig-bigint-fix.ps1`](../../tools/release/vendor-leansig-bigint-fix.ps1) —
  clones pin into `bazalinacaklar/leanSig-patched` (gitignored) and applies this patch
- [`../../tools/release/check-leansig-backend.ps1`](../../tools/release/check-leansig-backend.ps1) —
  writes gitignored `.cargo/config.toml` `[patch]`, runs check (optional `-Test`), restores
  `Cargo.lock`
- [`../../.cargo/config.toml.example`](../../.cargo/config.toml.example) — manual patch template

## Build

```powershell
tools/release/vendor-leansig-bigint-fix.ps1
tools/release/check-leansig-backend.ps1 -Test
```

Do not vendor the full leanSig tree into git. Prefer upstream fixing the pin.
Do not commit `.cargo/config.toml` or a path-patched `Cargo.lock`.
