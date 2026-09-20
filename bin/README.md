# bin/

Executable packages for Ethean.

| Path | Package | Role |
| --- | --- | --- |
| [ethean](./ethean/) | `ethean` | Production Lean Consensus Client binary |
| [ethean-leanvm-mock](./ethean-leanvm-mock/) | `ethean-leanvm-mock` | Dev/CI ELVM IPC mock prover (not production) |

`cargo build -p ethean` refreshes a PATH shim in `~/.cargo/bin` so the command
is simply `ethean` on Windows and Linux (no `install.sh`).
