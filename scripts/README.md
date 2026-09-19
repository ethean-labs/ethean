# scripts

Developer and CI helper scripts for the Ethean Lean Consensus workspace.

Keep scripts English, non-interactive, and safe for `cargo` + PowerShell on Windows.

## `ethean` on PATH (no install.sh)

`cargo build -p ethean` (debug or release) runs `bin/ethean/build.rs`, which writes a
shim into `$CARGO_HOME/bin` (usually `~/.cargo/bin`):

| Host | Shim |
| --- | --- |
| Windows | `ethean.cmd` |
| Linux / macOS | `ethean` (executable shell script) |

The shim prefers `target/release/ethean`, then `target/debug/ethean`, under this
workspace. After a successful build:

```bash
ethean version
ethean start --ticks 3
```

Rustup normally puts Cargo's `bin` on `PATH`. If the command is not found, add
`~/.cargo/bin` (Windows: `%USERPROFILE%\.cargo\bin`) and open a new shell.

Do not use a separate `install.sh` / `cargo install` step for day-to-day runs.
