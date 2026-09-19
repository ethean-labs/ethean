# Local private PQ mesh (dial generated nodes list)

Date: 2026-09-20

## Goal

Match the Ream / ethlambda / lean-quickstart pattern:

1. Build a **private** mesh for this run
2. Persist peer A’s dialable multiaddr as the mesh “nodes” file
3. Peer B **dials** that file (Ethean equivalent of `nodes.yaml`)

This does **not** join a public pq-devnet-4/5. It proves Status/dial wiring works
the same way peer clients do before an operator paste arrives.

## Scripts

| Script | Host |
| --- | --- |
| `scripts/local-pq-mesh.ps1` | Windows |
| `scripts/local-pq-mesh.sh` | Unix |

```powershell
.\scripts\local-pq-mesh.ps1
.\scripts\local-pq-mesh.ps1 -Network pq-devnet-4 -PeerBTicks 15
```

```bash
./scripts/local-pq-mesh.sh
NETWORK=pq-devnet-4 PEER_B_TICKS=15 ./scripts/local-pq-mesh.sh
```

## Artifacts (`target/local-pq-mesh/`)

| File | Role |
| --- | --- |
| `nodes.multiaddrs` | Generated peer list (one QUIC multiaddr; loopback-rewritten) |
| `peer-a.log` / `.err` | Listener (`--until-signal`) |
| `peer-b.log` / `.err` | Dialer (`--bootnodes` + wall-clock ticks) |

## Success signals in logs

- Peer B: `dialed bootnode`
- Either peer: `Status handshake` (when Status req/resp completes)

## Later: operator mesh

Paste a real run’s multiaddrs into `config/networks/pq-devnet-4.bootnodes` (or
`--bootnodes`) — same dial path, different address source.
