# leanVM Windows overlays

leanVM `e2592df4` (pq-devnet-4 pin) does not compile on Windows: `system-info`
calls `libc::getrusage`, and `zk-alloc` uses anonymous `mmap`.

These path crates are wired through `[patch."https://github.com/leanEthereum/leanVM.git"]`
in the workspace root `Cargo.toml` so Linux/macOS keep upstream behaviour while
Windows builds use:

- `peak_rss_bytes` via `GetProcessMemoryInfo`
- Arena disabled (`enable_arena` is a no-op); System allocator only
- `VirtualAlloc` stubs so `zk-alloc` links (arena path is not engaged)

Do not use this overlay as a fork of leanMultisig semantics — only OS glue.
