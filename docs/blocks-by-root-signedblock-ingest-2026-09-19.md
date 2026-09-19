# Blocks-by-root SignedBlock ingest (2026-09-19)

## What landed

Closes the open half of **C2** for pq-devnet-5 catch-up:

1. **Response codec** (`ethean-network` `reqresp/blocks_response.rs`): length-prefixed
   `count || (len || bytes)*` for blocks-by-root response bodies (unambiguous multi-block).
2. **Serve path**: QuicSwarm inbound replies use the same encoder.
3. **Ingest path** (`ethean-node` `blocks_sync.rs`): duty network pumps decode responses,
   route each blob through `IngestGossip` on synthetic topic
   `/leanconsensus/sync/block/ssz_snappy` (reuses Type-2 `SignedBlock` / `Block` decode + STF).
4. **Serve cache**: successful local proposal publish and successful sync decode call
   `put_block_bytes` so peers can fetch our head.

## Why

After Status head-gap, Ethean already flushed blocks-by-root requests. Responses were only
logged. Without decode/import, a dialed mesh peer cannot advance head from req/resp — a
blocker for operator pq-devnet-5 join even when bootnodes exist.

## Still open

- Multi-hop catch-up (parent chain) when remote head is many slots ahead — single-root
  Status gap fetch still applies only when parent matches local head.
- Operator fork digest + bootnodes for a live run.
- Crypto gates (leanSig / leanVM Type-2 prove-verify-split) remain fail-closed.

## Tests

- `ethean-network` `blocks_response` round-trip
- `ethean-node` `blocks_sync::ingests_signed_block_from_response`
