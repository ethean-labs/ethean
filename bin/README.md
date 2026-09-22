# bin/

Executable packages for Ethean.

| Path | Package | Role |
| --- | --- | --- |
| [ethean](./ethean/) | `ethean` | Production Lean Consensus Client binary |
| [ethean-prover](./ethean-prover/) | `ethean-prover` | leanMultisig prover process (Type-1 aggregation, Type-2 merge and split) |

`cargo build -p ethean` refreshes a PATH shim in `~/.cargo/bin` so the command
is simply `ethean` on Windows and Linux (no `install.sh`).
