# Blocks-by-root request_response wire (2026-09-19)

## What landed

QuicSwarm now runs a second Lean `request_response` behaviour for
`/leanconsensus/req/blocks_by_root/1/ssz_snappy` (framed Snappy), alongside Status.

After a Status head gap, the duty network path stages and **flushes** a
blocks-by-root outbox request. Pump events expose
`BlocksByRootRequest` / `BlocksByRootResponse`. Inbound requests are answered
from an in-memory `root → block bytes` cache (`put_block_bytes`).

Shared framed I/O lives in `quic_framed.rs` so Status and blocks codecs stay small.

## Still open

- Decode response bodies into `SignedBlock` and run STF / fork-choice import.
- Persist served blocks into the QuicSwarm cache when local proposals land.
- Operator fork digest + bootnodes still required for live pq-devnet-5 join
  ([leanroadmap.org](https://leanroadmap.org/) still lists D5 as Planned).

## Plan mapping

Closes wire half of **C2** in `bazalinacaklar/pq-devnet-5-ethean-plan-2026-09-19.md`
(fetch stream). Import/decode remains.
