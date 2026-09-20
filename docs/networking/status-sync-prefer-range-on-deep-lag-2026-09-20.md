# Status sync prefers blocks-by-range on deep lag (2026-09-20)

## What landed

After Status handshake, the node no longer stages **both** blocks-by-root and
blocks-by-range for every slot gap:

- Lag ≥ `RANGE_PREFER_LAG_SLOTS` (**4**, same as duty sync-lag threshold) →
  **blocks-by-range only**
- Smaller lag / head mismatch → **blocks-by-root only** (parent walk)

Parent multi-hop catch-up via `prepare_blocks_by_root_for_roots` is unchanged
(orphan drain still uses root fetches).

## Why

QuicSwarm range stream + ingest were already live; dual staging wasted bandwidth
and tracker slots on deep catch-up.

## Still open

| Gap | Notes |
| --- | --- |
| Hive / leanSpec fixture consumer | Matrix inclusion |
| A2/A3 live fork-digest + bootnodes | Operator values |
| Real leanVM binary (not mock) | Production SNARK |
| Upstream leanSig `num-bigint` 0.5 | B1 residual |
