# How Ream, ethlambda, and Zeam run Lean pq-devnets (2026-09-20)

Snapshot of public READMEs, lean-quickstart, leanroadmap (May 2026 stamp),
pq-devnet-4 plan, and client blogs. Protocol authority is still leanSpec, not
these clients.

## What a pq-devnet is

A **pq-devnet is a protocol generation**, not a public always-on Ethereum
testnet with baked-in bootnodes. Each generation adds one slice of Lean:

| Gen | What landed | Clients on the card |
| --- | --- | --- |
| 0 | leanSpec skeleton, 4s slots, QUIC, Gossipsub v1, modified 3SF-mini, no PQ sigs | Ream, Zeam, Qlean |
| 1 | leanSig sign/verify | + Lantern, Grandine |
| 2 | leanMultisig aggregation | + ethlambda |
| 3 | Separate aggregator role; aggregate gossip | same set |
| 4 | Recursive aggregation via leanVM; one aggregate per `attestation_data`; proposer keys | + gean, Peam |
| 5 | Type-2 / MultiMessageAggregate (one block-level proof); roadmap also lists Goldfish | same set; engineering ahead of the “Planned” card |
| 6 | Not specified. ethlambda candidates: Goldfish / RLMD-GHOST / trailing finality, or EL | planning |

Slot time stays **4 seconds**. Consensus in D0–D5 runs is still **3SF-mini**.
leanroadmap lists Goldfish under D5; ethlambda and leanSpec PR #717 treat
Goldfish as later. Treat Goldfish as spec-gated.

There is **no** public `pq-devnet-5.md` under leanEthereum/pm. Last official
plan file is [pq-devnet-4.md](https://github.com/leanEthereum/pm/blob/main/breakout-rooms/leanConsensus/pq-interop/pq-devnet-4.md).
Live ENRs live in operator `nodes.yaml`, not on leanroadmap.

## Shared join path (all three)

They do not dial a permanent public mesh from an empty peer file. A run is:

1. Write `validator-config.yaml` (IPs, QUIC ports, aggregator flags, key
   material).
2. Generate genesis + SSZ/JSON + hash-sig keys + `nodes.yaml` ENRs +
   `validators.yaml` + `config.yaml`.
3. Start every process with the **same** generated bundle:

```text
--network $configDir/config.yaml
--validator-registry-path $configDir/validators.yaml
--bootnodes $configDir/nodes.yaml
```

4. At least one node `--is-aggregator`. Without that, blocks may still be
   produced and the chain will not finalize.
5. Gossip topics `/leanconsensus/{fork}/{block|attestation|aggregate}/ssz_snappy`.
6. Req/resp: `/leanconsensus/req/status/1/ssz_snappy`,
   `blocks_by_root`, `blocks_by_range`. Transport is QUIC.

The shared harness is [blockblaz/lean-quickstart](https://github.com/blockblaz/lean-quickstart)
(`spin-node.sh --node zeam_0,ream_0,ethlambda_0 --generateGenesis`). Remote
interop uses the same YAML plus Ansible, leanpoint, Nemo, Prometheus.

## Ream ([ReamLabs/ream](https://github.com/ReamLabs/ream))

Rust workspace, **Beacon leftovers plus Lean**. Binary `ream lean_node`.
Docs: [ream.rs](https://ream.rs).

Lean crates sit beside Beacon crates: `crates/common/{chain,consensus,fork_choice,validator,checkpoint_sync}/lean`,
`crates/rpc/lean`, `crates/networking/{p2p,req_resp,syncer}`,
`crates/crypto/post_quantum`. Cargo.toml pins leanSig `devnet4` branch and
several leanVM / leanMultisig revs, including a Type-2 `lean-multisig-type2`
git pin.

Runtime:

- libp2p QUIC + Gossipsub.
- Lean discovery is **static bootnodes**. `lean_peers.yaml` on `master` is
  empty; `--bootnodes default` does not join a public D4.
- Dual validator keys (attestation + proposer) from D4.
- `--is-aggregator`, subnet ids, `--attestation-committee-count`,
  `--block-production` (round-based vs tiered).
- Checkpoint sync via `--checkpoint-sync-url`.
- HTTP `/lean/v0/...` (local-pq-devnet curls `:5052`).

How they actually run a mesh:

| Mode | Tool |
| --- | --- |
| Local 6-node Ream-only | [ReamLabs/local-pq-devnet](https://github.com/ReamLabs/local-pq-devnet) (`setup-genesis.sh` then `docker-compose up`) |
| Kurtosis | [ReamLabs/pq-devnet-package](https://github.com/ReamLabs/pq-devnet-package) (Ream-first; Zeam/Qlean still TODOs there) |
| Mixed-client | lean-quickstart nightly / operator Ansible |
| Smoke | `--bootnodes none` or in-process loopback |
| Docs extra | `lean_node --network ephemery` — **not** pq-devnet |

## ethlambda ([lambdaclass/ethlambda](https://github.com/lambdaclass/ethlambda))

Lean-only Rust, small tree: `bin/ethlambda` plus `crates/{blockchain,common,net,storage}`.
BlockChain is an actor (`spawned`): sequential fork choice + STF + duties;
P2P/RPC stay async around it. Crypto wraps leanSig / leanMultisig. Storage is
RocksDB or in-memory.

README status (2026-09-20 fetch): **running pq-devnet-5 spec**, Docker tag
`devnet5`. Older `devnetX` tags are discontinued when the next gen ships.
`pq-devnet-6` is planning only.

Local mesh: `make run-devnet` clones lean-quickstart, builds the image, starts
all configured clients. Manual runs still need `--is-aggregator` on at least
one node. Optional discv5; gossip still uses the same Lean topics. Metrics
follow leanMetrics.

D5 meaning in their blog (16 Jun 2026): one **multi-message** block proof
instead of one leanVM proof per attestation (~200 KiB vs several MiB at 16
attestations). They keep 3SF-mini for D5 and put Goldfish / RLMD-GHOST /
trailing finality in the D6 narrative. A later post argues moving the
proposer signature **out** of the Type-2 proof to cut proposer latency
(protocol-breaking; needs interop agreement).

## Zeam ([blockblaz/zeam](https://github.com/blockblaz/zeam))

Zig 0.16 client. Distinct from the two Rust clients: they also prove the
**state transition** in a ZK-VM (risc0 and OpenVM). Packages:

- `state-transition` — Zig STF
- `state-transition-runtime` — RISC-V guest
- `state-proving-manager` — prove/verify orchestration
- `node`, `network` (libp2p Zig↔Rust), `api`, `cli`, `database` (RocksDB),
  `xmss`, `key-manager`, `spectest`, `metrics`

Local 2-node finalization fixtures live under `pkgs/cli/test/fixtures`.
Production interop is the **lean-quickstart submodule** they maintain.
Checkpoint sync: `--checkpoint-sync-url`. Discovery: static ENRs.

README “POC” pages still describe an older Beam proving slice; the live
`pkgs/` tree and lean-quickstart integration are the Lean interop client
used on D0–D5 cards.

## Comparison (runtime, not style)

| | Ream | ethlambda | Zeam |
| --- | --- | --- | --- |
| Language | Rust 1.98 | Rust 1.97 | Zig 0.16 (+ Rust for ZK bindings) |
| Scope | Beacon + Lean | Lean only | Lean + STF ZK proofs |
| Default D-gen (public claim) | D4 leanSig pin + Type-2 crate | **D5** image `devnet5` | D-gen via quickstart `--tag` |
| Discovery | static bootnodes | optional discv5 | static ENRs |
| Harness they own | local-pq-devnet, Kurtosis package | `make run-devnet` | lean-quickstart |
| Aggregator | `--is-aggregator` | same; required for finality | same; quickstart picks one node |
| Checkpoint | HTTP | HTTP (built later than D2 blog) | HTTP |

## Mapping to Ethean

Ethean already follows the same join contract: generated or pasted bootnodes,
not a public D4/D5 ENR list. Operational default in this repo is
`--network pq-devnet-4` with D5 files kept ready. Match peers on wire
(SSZ, topics, Status, Type-2 body) from leanSpec; do not copy crate layout.

Related notes: [how-ream-connects-without-public-bootnodes-2026-09-20.md](how-ream-connects-without-public-bootnodes-2026-09-20.md),
[pq-devnet-5-research-refresh-2026-09-19.md](../pq-devnet/pq-devnet-5-research-refresh-2026-09-19.md),
[peer-reference-clients.md](../peer-reference-clients.md).
