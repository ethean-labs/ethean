# Dual mode: durable data-dir + ephemeral smoke (2026-09-20)

## Goal

Keep both operator paths:

1. **Fixed genesis** under `--data-dir` (write once, resume head).
2. **Ephemeral recent-genesis** smoke via `--ephemeral` (or no `--data-dir`).

## What landed (updated)

| Piece | Role |
| --- | --- |
| `crates/node/src/genesis_bundle.rs` | `ethean-genesis-v1` `genesis.json` |
| `crates/node/src/persist_ssz.rs` | `genesis.ssz`, `state.ssz`, `blocks/*.ssz` |
| `crates/node/src/chain_redb.rs` | `ethean.redb` |
| `crates/node/src/chain_persist.rs` | Open / flush / restore / `--reset-chain` |
| `crates/node/src/client_data_dir.rs` | Durable open + flush |

Files under `--data-dir PATH`:

- `genesis.json` — profile + full genesis state
- `genesis.ssz` / `state.ssz` / `head.root` / `blocks/<root>.ssz`
- `ethean.redb` — SSZ blobs in a local KV
- `log/ethean-YYYY-MM-DD-HHMMSS-log` — process log (removed with `--reset-chain`)

Legacy `genesis_pin.json` / `head_snap.json` are still read and migrated.

## Operator examples

```bash
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data
ethean start --until-signal --network pq-devnet-4 --ephemeral
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data --reset-chain
```

See [ethean-redb-ssz-data-dir-2026-09-20.md](./ethean-redb-ssz-data-dir-2026-09-20.md)
and [reset-chain-wipes-data-dir-2026-09-20.md](./reset-chain-wipes-data-dir-2026-09-20.md).
