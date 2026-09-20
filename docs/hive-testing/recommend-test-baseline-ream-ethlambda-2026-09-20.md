# Recommended test baseline: Ream ops + ethlambda protocol (2026-09-20)

## Verdict

Start day-to-day Ethean testing on the **Ream / lean-quickstart join pattern**
(private mesh, generated peer list, dial). Use **ethlambda** as the primary
**Lean-only Rust** reference when a protocol or crate-shape question comes up.
Do **not** base Ethean’s layout or language on Zeam; use Zeam only later as a
lean-quickstart peer in a mixed mesh.

## Why this split

| Need | Pick | Reason |
| --- | --- | --- |
| How to run and dial today | Ream-style local mesh | Ethean already has `scripts/local-pq-mesh.*`; Ream’s empty public bootnodes match our reality |
| Which Lean Rust client to read | ethlambda | Lean-only, small crate tree, explicit D5 claim, Type-2 blogs |
| Mixed-client interop later | lean-quickstart (+ Zeam as a peer) | Shared genesis/`nodes.yaml`; Zeam owns the harness, not our style |
| Network label for smoke | `pq-devnet-4` | Operational default; keep D5 files ready |
| Goldfish | Skip until spec | ethlambda frames it as D6; leanroadmap D5 card is not enough |

## Start this week (ordered)

1. **Local private mesh (Ethean ↔ Ethean)**  
   `.\scripts\local-pq-mesh.ps1` or `./scripts/local-pq-mesh.sh` with
   `--network pq-devnet-4`. Success: `dialed bootnode`, Status handshake in logs.

2. **Single-node offline smoke**  
   `ethean start --network pq-devnet-4 --ticks N` with empty bootnodes — expect
   WARN, not crash (same as Ream `--bootnodes none`).

3. **Protocol reads when stuck**  
   Open ethlambda paths for Status, gossip topics, aggregator flag, Type-2 body —
   match wire behavior, not their actor crate names.

4. **Optional mixed mesh (only after local Status is green)**  
   Clone lean-quickstart under `bazalinacaklar/` (gitignored), pin one
   `devnet4` or `devnet5` tag, add an Ethean binary/docker participant when we
   have a join script. Prefer one Ream or ethlambda peer first, not Zeam-first.

5. **Do not** paste empty public D4/D5 bootnode files and expect a live join.

## What not to copy

- Ream’s Beacon crate tree and dual-client workspace.
- Zeam’s Zig / ZK-VM proving layout as Ethean’s default architecture.
- Any peer’s house style, comments, or module names.

Authority order stays: leanSpec pin → compatibility ledger → peer wire behavior.

## Related

- [peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](../peer-clients/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
- [local-pq-mesh-private-dial-2026-09-20.md](../pq-devnet/local-pq-mesh-private-dial-2026-09-20.md)
- [default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](../pq-devnet/default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md)
