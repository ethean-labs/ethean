# Durable data-dir: genesis.json, SSZ, and ethean.redb (2026-09-20)

`--data-dir PATH` now keeps an Ethean chain package, not a tiny JSON pin.

## Layout

| File | Role |
| --- | --- |
| `genesis.json` | `ethean-genesis-v1` operator document: fork/profile, validator count, `state_root`, full genesis `state` |
| `genesis.ssz` | SSZ-encoded genesis `State` |
| `state.ssz` | Current head `State` (includes `historical_block_hashes`) |
| `head.root` | 32-byte head root as hex |
| `blocks/<root>.ssz` | Signed block payload when a proposal is flushed |
| `ethean.redb` | Local KV: schema, genesis SSZ, head root, states, blocks |
| `log/ethean-YYYY-MM-DD-HHMMSS-log` | Process tracing for that start (stdout still prints) |

`--reset-chain` deletes chain files (and leftover `genesis_pin.json` / `head_snap.json`).
It does **not** delete `log/`.

Resume order: `ethean.redb` → `state.ssz` + `head.root` → legacy JSON snapshot.

## Why this shape

Consensus history belongs inside SSZ `State`, not a growing hex JSON array.
`genesis.json` stays human-readable for operators; `ethean.redb` and `.ssz` files
are the durable store.

## Code

- `crates/node/src/persist_paths.rs` — names
- `crates/node/src/genesis_bundle.rs` — `genesis.json`
- `crates/node/src/persist_ssz.rs` — SSZ files
- `crates/node/src/chain_redb.rs` — `ethean.redb`
- `crates/node/src/chain_persist.rs` — open / flush / restore
- `bin/ethean/src/file_log.rs` — dated `log/ethean-*-log` files
