# Local proving devnet on one host (2026-09-29)

A multi-node Ethean mesh with real PROD XMSS keys and the leanVM prover, on a
single Windows machine, without Docker. It exists to measure mesh timing and
finality that the solo `--local-finality` path cannot show (gossip, the
aggregator role, block proofs racing other proposers).

## Pieces

- `ethean devnet-init` (`crates/node/src/devnet_bundle.rs`,
  `bin/ethean/src/devnet_init.rs`) writes a lean-quickstart shaped bundle:
  `config.yaml` (GENESIS_TIME, GENESIS_VALIDATORS attestation/proposal keys),
  `validators.yaml` (node id -> index / pubkey / privkey file),
  `hash-sig-keys/validator_{i}_{attestation,proposal}_sk.ssz`, `nodes.yaml`
  (QUIC multiaddrs) and one secp256k1 `ethean_k.key` per node. Keys are
  generated once (`key_gen(0, 131072)`, about 20 s each) and reused; only the
  genesis time changes between runs. It prints one `ethean start …` line per
  node; node 0 gets `--is-aggregator` (nodes `0..A` with `--aggregators A`).
- `scripts/local-devnet.ps1` builds release `ethean` + `ethean-prover`, runs
  `devnet-init`, starts the nodes (logs in `<out>/logs/`), polls
  `/lean/v0/fork_choice` on ports `5052+k` and prints head, safe target,
  justified and finalized slots, then stops the nodes.

```powershell
.\scripts\local-devnet.ps1                       # 3 nodes, 180 s
.\scripts\local-devnet.ps1 -Nodes 4 -RunSeconds 300 -DebugLog
.\scripts\local-devnet.ps1 -MaxBlockData 3       # passes --max-block-attestation-data (default 1)
# node 3 joins at genesis+90 s from node 0's finalized checkpoint,
# node 2 is killed at +190 s and restarted from its data dir 20 s later
.\scripts\local-devnet.ps1 -Nodes 4 -RunSeconds 330 -LateJoin 3 -LateJoinAt 90 -Restart 2 -RestartAt 190 -RestartDown 20
```

Node 0 cannot be a scenario node (it serves the checkpoint and aggregates).
The restarted node logs to `ethean_k.restart.{out,err}.log`.

Ports: QUIC `9000+k`, HTTP `5052+k`, metrics `9200+k`. Output defaults to
`target/local-devnet` (gitignored with `target/`).

## What the first runs found

The first 150 s run with 3 nodes had blocks propagating but justified stuck at
0, the aggregator never publishing a block, and one node logging a Status retry
every second. Five bugs were behind it; see
[mesh-finality-fixes-2026-09-29.md](../lean-spec/mesh-finality-fixes-2026-09-29.md).

After the fixes (3 nodes, 1 validator each, default data cap 3, 20-core host):

| Metric | Value |
| --- | --- |
| Justified progression | 4 → 6 → 9 → 25 → 30 |
| First finalization | slot 25 (at about slot 38), same on all three nodes |
| Block proof jobs | 42, one dropped as stale |
| Status retries to a live peer | 0 (was about 2100 in 150 s) |
| WARN / ERROR lines | 1 (the stale proof) |

Heads still skip slots: a block proof takes several seconds at the
`e2592df4` leanVM pin, so blocks regularly land one to three slots late and
justification advances in jumps. That is the prover latency already recorded in
[prover-timing-block-data-cap-2026-09-29.md](../lean-crypto/prover-timing-block-data-cap-2026-09-29.md),
not a protocol fault.

Longer 300 s runs then stalled finality. The safe-target fix, the new default
cap of 1 and once-per-slot aggregation (after which finalized trails the head
by about 7 slots) are in
[safe-target-merge-block-data-cap-1-2026-09-29.md](../lean-spec/safe-target-merge-block-data-cap-1-2026-09-29.md).

The late-join and restart scenarios, and the sync fixes they needed, are in
[late-join-restart-mesh-2026-09-29.md](./late-join-restart-mesh-2026-09-29.md).

Several validators per node (`-ValidatorsPerNode`), several subnets
(`-Subnets`) and aggregators (`-Aggregators`) are in
[scale-multi-aggregator-2026-09-30.md](./scale-multi-aggregator-2026-09-30.md).

## Limits

- One host: no real network latency; all provers share the CPU, so proving
  times are worse than on separate machines.
- Only Ethean nodes. Multi-client runs (Ream, Zeam, ethlambda …) still need
  Hive or leanstart with Docker.
