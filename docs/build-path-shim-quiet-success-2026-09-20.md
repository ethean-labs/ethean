# Quiet PATH shim on successful build (2026-09-20)

## Problem

`cargo build -p ethean` printed:

```text
warning: ethean@0.1.0: ethean PATH shim ready at … (run: ethean version)
```

That line came from `bin/ethean/build.rs` using `println!("cargo:warning=…")`
after a successful shim write. Cargo surfaces every `cargo:warning=` as a
compiler warning, so a normal success looked like a build problem.

## Fix

- Keep writing the PATH shim on each build of the `ethean` package.
- Emit `cargo:warning=` only when the shim cannot be updated (permission /
  missing home, etc.).
- Do not print a success warning.

## Verify

```bash
cargo build -p ethean
# no "PATH shim ready" warning on success
ethean version
```
