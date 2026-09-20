# Peer clients: fixed genesis vs Ethean solo restart (2026-09-20)

## Question

Do Ream, Zeam, qlean-mini, ethlambda, Lantern, gean, and Peam ship a permanent
baked-in genesis, or regenerate every process start?

## Answer

They do **not** bake a public always-on genesis into the client binary for
pq-devnets. They generate a **shared genesis bundle once**, then every node
loads **that same on-disk bundle** for the life of the mesh.

Typical artifacts (lean-quickstart / Ream local-pq-devnet):

- `validator-config.yaml` (source of truth: IPs, ports, aggregator flags)
- `config.yaml` (includes `GENESIS_TIME`)
- `genesis.json` / `genesis.ssz`
- `validators.yaml` / `annotated_validators.yaml`
- `nodes.yaml` (ENRs)
- `hash-sig-keys/` (PQ keys; kept unless `--forceKeyGen`)

Harnesses:

| Client / harness | How genesis is produced |
| --- | --- |
| Shared | [blockblaz/lean-quickstart](https://github.com/blockblaz/lean-quickstart) `generate-genesis.sh` / `spin-node.sh --generateGenesis` |
| Ream | [ReamLabs/local-pq-devnet](https://github.com/ReamLabs/local-pq-devnet) `./setup-genesis.sh` then compose; Kurtosis / leanstart same idea |
| Zeam | lean-quickstart submodule; fixtures under `pkgs/cli/test/fixtures` for tiny local finality tests |
| ethlambda | `make run-devnet` → clones quickstart, **generates genesis files**, starts clients; RocksDB data dirs separate |
| qlean-mini, Lantern, gean, Peam | Same quickstart join contract (`--network` / validators / bootnodes); no separate public genesis |

`--generateGenesis` means: **fresh `GENESIS_TIME`** (and usually clean data
dirs). Without that flag, an existing bundle is reused. Restarting a node with
the **same** genesis dir + data dir continues the chain; regenerating genesis
starts a new chain.

## Contrast with Ethean

- **Ephemeral** (`--ephemeral` / no `--data-dir`): builds a **recent
  in-memory genesis** each process start. Head lives in RAM — Ctrl-C at slot 48
  then restart near slot 4–5 is expected.
- **Durable** (`--data-dir PATH`): pins `genesis_time` once and resumes
  `head_snap.json` (peer-like local analogue). See
  [dual-mode-persist-and-ephemeral-2026-09-20.md](./dual-mode-persist-and-ephemeral-2026-09-20.md).

Interop mesh import of a full lean-quickstart bundle remains a follow-up; wire
contract (bootnodes, topics, aggregator) already matches peers.

## Sources

- lean-quickstart README (`--generateGenesis`, `--forceKeyGen`, `--cleanData`)
- ReamLabs/local-pq-devnet README (`setup-genesis.sh`)
- ethlambda README (`make run-devnet` generates fresh genesis for that run)
- Prior note: [peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](./peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
