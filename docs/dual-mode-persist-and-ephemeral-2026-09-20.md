# Dual mode: durable data-dir + ephemeral smoke (2026-09-20)

## Goal

Keep both operator paths that peer Lean clients already use in practice:

1. **Peer-like fixed genesis** under `--data-dir` (pin once, resume head).
2. **Ephemeral recent-genesis** smoke via `--ephemeral` (or no `--data-dir`).

## What landed

| Piece | Role |
| --- | --- |
| `crates/node/src/chain_snap.rs` | JSON DTOs for genesis pin + head state |
| `crates/node/src/chain_persist.rs` | Load/create pin, save/restore head, `--reset-chain` |
| `crates/node/src/client_data_dir.rs` | `open_data_dir_with_roles` uses fixed genesis + resume |
| `crates/node/src/local_genesis.rs` | `fixed_devnet_genesis(validators, genesis_time)` |
| CLI | `--ephemeral`, `--reset-chain`; `--data-dir` now resumes |
| Duty loops | Flush `head_snap.json` after each wall/mesh step |

Files under `--data-dir PATH`:

- `genesis_pin.json` — `genesis_time`, `validators`, `seconds_per_slot`
- `head_snap.json` — head root + local state (justified/finalized included)

RocksDB is optional. If `ethean-storage/rocksdb` is not enabled, the node still
uses the JSON snapshots for resume and keeps an in-memory store.

## Operator examples

```bash
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data
ethean start --until-signal --network pq-devnet-4 --ephemeral
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data --reset-chain
```

## Peer model (context)

Ream / Zeam / ethlambda / qlean-mini / Lantern / gean / Peam generate a shared
bundle once (`config.yaml`, genesis SSZ, `nodes.yaml`, keys) and reuse it.
Ethean’s `--data-dir` is the local solo analogue of that pin; full lean-quickstart
bundle import remains a follow-up for multi-client mesh.

See also: [peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](./peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md).
