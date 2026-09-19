# Blocks-by-range request scaffold (2026-09-19)

## What landed

Parent-walking via blocks-by-root works for modest gaps. Deep Status lag needs a
contiguous slot fetch. This lands the **request codec only**:

- `blocks_by_range_for_status_gap(local_head_slot, remote)` → start = local+1,
  count = min(lag, MAX_BLOCKS_PER_REQUEST), step = 1
- `encode_blocks_by_range` / `decode_blocks_by_range` (3× u64 LE)
- Protocol id: `/leanconsensus/req/blocks_by_range/1/ssz_snappy`

## Still open

- QuicSwarm `request_response` behaviour + framed codec (mirror blocks-by-root)
- Slot→SignedBlock serve cache for replies
- Duty-network policy: prefer range when lag ≫ 1, else single-root parent walk
- Response ingest into `blocks_sync` / orphan drain

## Plan mapping

Advances remaining **C2** deep-catch-up work after multi-hop parent walk.
