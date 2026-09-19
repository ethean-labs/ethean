# bin/

Executable packages for Ethean.

| Path | Package | Role |
| --- | --- | --- |
| [ethean](./ethean/) | `ethean` | Production Lean Consensus Client binary |

`cargo build -p ethean` refreshes a PATH shim in `~/.cargo/bin` so the command
is simply `ethean` on Windows and Linux (no `install.sh`).
