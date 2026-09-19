# Blocks-by-range Quic stream (pq-devnet-5 C2)

Date: 2026-09-19

## What landed

Lean `blocks_by_range` is now a live QuicSwarm request/response path, not only a codec/scaffold:

- Codec + behaviour already under `quic_range_codec` (`/leanconsensus/req/blocks_by_range/1/ssz_snappy`).
- `QuicSwarm` pumps `BlocksByRange` events; inbound requests are answered from a slot→bytes serve cache.
- `SwarmFacade` stages/flushes a dedicated `blocks_range_outbox` and exposes `put_block_at_slot`.
- After Status handshake, the node stages both blocks-by-root (head) and blocks-by-range (slot gap).
- Pump budget collects range responses; duty loop ingests them with the same SSZ blob decoder as root sync.
- Published proposals and ingested sync blocks are indexed by slot for later range replies.

## Key modules

| Area | Files |
| --- | --- |
| Bind split | `quic_swarm_bind.rs`, `quic_swarm.rs` |
| Events / reply | `quic_events.rs` (`reply_blocks_by_range`) |
| Outbound stage | `reqresp/range_outbound.rs` |
| Facade | `swarm.rs` + `swarm_range.rs` |
| Node | `status_handshake.rs`, `duty_network.rs`, `swarm_pump.rs`, `blocks_sync.rs` |

## Response shape

Range replies reuse the scaffold blocks-by-root response encoding (`u32` count + length-prefixed blobs). Callers that already decode root responses can ingest range payloads unchanged.

## Still open (related)

- B1 leanSig production verify, leanVM Type-2 SNARK prove/verify (fail-closed).
- C3 operator fork-digest gossip isolation.
- Live operator bootnodes for a public D5 mesh.
