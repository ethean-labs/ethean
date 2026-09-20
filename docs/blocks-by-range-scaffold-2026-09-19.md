# Blocks-by-range request scaffold (2026-09-19)

## What landed

Parent-walking via blocks-by-root works for modest gaps. Deep Status lag needs a
contiguous slot fetch. This lands the **request codec only**:

- `blocks_by_range_for_status_gap(local_head_slot, remote)` → start = local+1,
  count = min(lag, MAX_BLOCKS_PER_REQUEST), step = 1
- `encode_blocks_by_range` / `decode_blocks_by_range` (3× u64 LE)
- Protocol id: `/leanconsensus/req/blocks_by_range/1/ssz_snappy`

## Still open

- Duty-network policy: prefer range when lag ≫ 1 — **landed**
  ([status-sync-prefer-range-on-deep-lag-2026-09-20.md](./status-sync-prefer-range-on-deep-lag-2026-09-20.md))
- QuicSwarm stream + ingest — **landed**
  ([blocks-by-range-quic-stream-2026-09-19.md](./blocks-by-range-quic-stream-2026-09-19.md))

## Plan mapping

Advances remaining **C2** deep-catch-up work after multi-hop parent walk.
