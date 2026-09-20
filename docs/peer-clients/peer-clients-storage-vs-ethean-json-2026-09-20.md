# Peer client storage vs Ethean JSON snapshots (2026-09-20)

Peers do **not** persist chain history as a pretty-printed
`head_snap.json` with a hex `historical_block_hashes` array.

That list is a Lean `State` field. Ream, Zeam, qlean-mini, ethlambda, Lantern,
gean, and Peam keep it **inside SSZ-encoded state**, then write the blob to an
embedded DB or to `.ssz` files under `--data-dir`.

## What Ethean writes today

`--data-dir ./test-devnet` currently produces only:

- `genesis_pin.json`
- `head_snap.json`

No RocksDB/`*.sst` and no `state.ssz`. Default `ethean` does not enable
`ethean-storage/rocksdb`; the duty loop flushes JSON only.

## What peers write under `--data-dir`

| Client | Store | On-disk shape |
| --- | --- | --- |
| Ream | `redb` | `<data-dir>/ream.redb` (SSZ tables). If `--data-dir` is omitted, OS app data via `ProjectDirs`. |
| Zeam | RocksDB / LMDB | Column families for blocks, states, checkpoints |
| qlean-mini | RocksDB | KV spaces under `--data-dir` |
| ethlambda | RocksDB | SSZ `State` / blocks in tables |
| Lantern | filesystem | `state.ssz`, `blocks/<root>.ssz`, `states/<root>.ssz` |
| gean | Pebble | `--data-dir` (default `./data`) |
| Peam | `redb` + blobs | `canonical.redb` + SSZ blobs (`LEANSTRG`) |

Genesis is a **separate** shared bundle (`genesis.ssz`, `config.yaml`, keys)
from lean-quickstart. Data dir is per-node chain history.

Related: [peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md](./peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md),
[dual-mode-persist-and-ephemeral-2026-09-20.md](./dual-mode-persist-and-ephemeral-2026-09-20.md).
