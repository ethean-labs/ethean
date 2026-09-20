# How Ream tests and connects without public bootnodes (2026-09-20)

Ream does **not** dial a permanent public pq-devnet from an empty
`lean_peers.yaml`. It connects in these modes:

## 1. Local PQ mesh they generate themselves

[ReamLabs/local-pq-devnet](https://github.com/ReamLabs/local-pq-devnet):

1. `./setup-genesis.sh` — creates keys, genesis, and peer list for that run
2. `docker-compose up` — starts ~6 Ream lean nodes on a private Docker net
3. Each node gets the **generated** `nodes.yaml` as `--bootnodes`

Same pattern as [blockblaz/lean-quickstart](https://github.com/blockblaz/lean-quickstart):
generate `genesis/nodes.yaml`, pass `--bootnodes $configDir/nodes.yaml`.

## 2. Multi-client CI (lean-quickstart fork)

Nightly matrix (example in Ream workflows) clones/patches lean-quickstart and
starts every client with:

```text
--network $configDir/config.yaml
--validator-registry-path $configDir/validators.yaml
--bootnodes $configDir/nodes.yaml
```

Peers find each other because **all nodes share the same freshly generated ENR list**,
not because Ream embeds public IPs.

## 3. Single-node / API smoke

Lean API CI often uses `--bootnodes none` (no mesh). Unit/integration tests dial
loopback peers they spawn in-process. No public D4 required.

## 4. Docs “quick start” (Ephemery)

`ream lean_node --network ephemery` is a separate lean-ephemery path in their docs.
That is **not** “paste empty lean_peers and join public pq-devnet-4”.

## Mapping to Ethean

| Ream | Ethean |
| --- | --- |
| `setup-genesis` + `nodes.yaml` | two-process local mesh / future local-pq helper |
| `--bootnodes /path/nodes.yaml` | `--bootnodes` / `config/networks/pq-devnet-4.bootnodes` |
| `--bootnodes none` / empty default | offline smoke WARN (what you already see) |
| Operator interop paste | same: fill the bootnodes file for that run |

Bottom line: Ream “connects” by **creating or receiving a private peer list for that run**,
then dialing it. Ethean is at the same stage until someone pastes a live `nodes.yaml`.
