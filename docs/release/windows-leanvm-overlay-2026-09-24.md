# Windows leanVM overlay for release archives (2026-09-24)

## Why

`v0.1.53` release-binaries failed on `x86_64-pc-windows-msvc` because leanVM
`e2592df4` `system-info` calls `libc::getrusage` and `zk-alloc` uses anonymous
`mmap`. Operators saw Linux/macOS rows only under **Binaries**.

## What landed

| Piece | Role |
| --- | --- |
| `vendor/leanvm-windows/system-info` | `peak_rss_bytes` via `GetProcessMemoryInfo` on Windows |
| `vendor/leanvm-windows/zk-alloc` | Windows syscall stubs; arena left disabled (System alloc) |
| Root `[patch."…/leanVM.git"]` | Path overlays for those two crates |
| Workflow | Windows job no longer `experimental: true` |

Plonky3 stays pinned at `3f67d136` in `Cargo.lock` (do not `cargo generate-lockfile`
without re-pinning — newer Plonky3 mixes `num-bigint` 0.4/0.5 and breaks leanSig).

## Verify

```powershell
cargo build -p ethean -p ethean-prover --release --locked
```
