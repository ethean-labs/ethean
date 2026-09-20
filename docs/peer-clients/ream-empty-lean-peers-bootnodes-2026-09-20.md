# Ream has no public pq-devnet-4 bootnodes either (2026-09-20)

## Symptom

`ethean start --network pq-devnet-4` logs:

```text
WARN … no bootnodes configured; offline smoke is expected …
Duty loop finished ticks_accepted=…
```

That is **not a crash**. QuicSwarm binds, health is ok, duties run offline.

## What Ream does

Inspected [ReamLabs/ream](https://github.com/ReamLabs/ream):

| Path | Role |
| --- | --- |
| `crates/networking/p2p/src/bootnodes.rs` | Lean `--bootnodes default` → `to_multiaddrs_lean()` |
| `crates/networking/p2p/resources/lean_peers.yaml` | **Empty file (0 bytes) on `master`** |
| Beacon `bootnodes_*.yaml` | Eth mainnet/sepolia/hoodi only — not Lean pq-devnet |

Lean CI / lean-quickstart pass **`--bootnodes $configDir/nodes.yaml`** generated per local or operator interop run. There is no baked-in public D4 multiaddr list in Ream.

## How to get a mesh anyway

1. **Local two-process** (same host):
   - Terminal A: `ethean start --until-signal --network pq-devnet-4`
   - Copy logged `dialable=…`, rewrite host to `127.0.0.1`
   - Terminal B: `ethean start --network pq-devnet-4 --ticks 3 --wall-clock --bootnodes '<that-addr>'`
2. **Operator / lean-quickstart**: take `genesis/nodes.yaml` ENRs, convert to QUIC multiaddrs, paste into `config/networks/pq-devnet-4.bootnodes`.
3. Do **not** expect Ream’s empty `lean_peers.yaml` to dial a public D4.

## Related

- [default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](../pq-devnet/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md)
- [pq-devnet-5-client-run-2026-09-19.md](../pq-devnet/pq-devnet-5-client-run-2026-09-19.md) (same two-process pattern)
