# Serve only verified sync blocks (2026-11-02)

Peer note: ethlambda [#545](https://github.com/lambdaclass/ethlambda/issues/545)
(blocks stored as pending and served before validation). Same risk in
`ingest_blocks_by_root_response`: the req/resp serve cache was filled as soon
as SSZ decoded, before `import_decoded_block`.

## Change

The cache (`put_block_bytes` / `put_block_at_slot`) and durable queue are
updated only after a successful import. Orphans stay in the local parent-wait
map and are not advertised on blocks-by-root or blocks-by-range. Draining an
orphan into the chain also seeds the cache at that point.

Type-1 aggregation is not started while a block proof is already in flight, so
interval-2 jobs do not take the prover away from a late previous-slot block
(gean yield-to-proposal / local `prover-contention-proposer` note). Native
leanVM jobs are still not cancellable.

## Tests

`never_advances_head_without_a_verified_block` now also checks that the durable
queue stays empty.
