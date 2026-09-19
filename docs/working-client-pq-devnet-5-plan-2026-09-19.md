# Working-client plan: pq-devnet-5 target (2026-09-19)

## Verdict

pq-devnet-5 is the **current generation target**. leanroadmap still labels the card **Planned**, but leanSpec Type-2 work is merged and peers have been running private interop. There is **no** finished public mesh with published bootnodes. Ethean defaults `start` to that **network label** and dials only operator-supplied QUIC multiaddrs.

Full research summary: [pq-devnet-5-research-refresh-2026-09-19.md](./pq-devnet-5-research-refresh-2026-09-19.md).

## Peer baseline

| Name | Use |
| --- | --- |
| Zeam | Primary Zig reference (devnet5 release shape, static nodes/ENR) |
| Ream | Primary Rust reference (`devnet5` feature, `--bootnodes`, Hive) |
| ethlambda | Type-2 performance / interop notes |
| Peam | Secondary; older aggregation surfaces — not pin authority |
| Beam | Historical Lean name only |

## Sprint landings

1. CLI `--network pq-devnet-5` (default) + `--bootnodes` + `ETHEAN_BOOTNODES` + `config/networks/pq-devnet-5.bootnodes`
2. Dial bootnodes after QuicSwarm bind (feature `libp2p-quic`)
3. Root README Quick Start rewritten for real `ethean` binary
4. Clear offline warning when bootnode list is empty

## Follow-ups (D5-focused)

- Fork digest override landed; still need the operator/leanSpec pin value for a live run.
- leanSig + leanVM production gates (**Type-1 merge + Type-2 prove/verify/split**).
- Status / blocks-by-root stream exchange — Status + blocks-by-root wire **and SignedBlock ingest** landed; multi-hop parent catch-up still open ([blocks-by-root-signedblock-ingest-2026-09-19.md](./blocks-by-root-signedblock-ingest-2026-09-19.md)).
- Decode/validate D5 block body: single Type-2 proof + re-agg cache from split.
- Hive / leanSpec fixture consumer for matrix inclusion.
- Goldfish / PQ heartbeat **only** when leanSpec for the run requires it (may be D6).

## Sprint landings (Status sync)

5. Boot stores local Status, queues `StatusSessionBook`, stages Status/blocks outboxes
6. See [status-handshake-outbox-2026-09-19.md](./status-handshake-outbox-2026-09-19.md)
7. QuicSwarm Status request_response send/receive — [quic-status-reqresp-wire-2026-09-19.md](./quic-status-reqresp-wire-2026-09-19.md)
8. Local client verification (offline + two-process Status mesh) — [pq-devnet-5-client-run-2026-09-19.md](./pq-devnet-5-client-run-2026-09-19.md)
9. Blocks-by-root request_response wire — [blocks-by-root-reqresp-wire-2026-09-19.md](./blocks-by-root-reqresp-wire-2026-09-19.md)
10. Blocks-by-root SignedBlock ingest — [blocks-by-root-signedblock-ingest-2026-09-19.md](./blocks-by-root-signedblock-ingest-2026-09-19.md)

## How to run (summary)

See root README Quick Start and [deployment.md](./deployment.md).
