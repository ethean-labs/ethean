# Local hive lean-simulator runs (2026-09-24)

Host: Linux, Docker daemon started for the session, hive built with
`-buildvcs=false`. Client image: local `ghcr.io/ethean-labs/ethean:devnet5`
built from the root `Dockerfile`; hive drop-in `clients/ethean/`.

```
./hive --sim lean --client ethean_devnet5 --client-file simulators/lean/clients/devnet5.yaml \
  --sim.buildarg devnet5_tag=0588c2d2 --results-root ./workspace/logs
```

`--sim.buildarg devnet5_tag=0588c2d2` pins the lean-spec helper to the
leanSpec commit before #1205: hive's `clients/lean-spec-client/lean_spec_runtime.py`
still imports `lean_spec.spec.crypto.merkleization`, which leanSpec `main`
no longer has, so with the default tag the helper dies with
`ModuleNotFoundError` for every client and the simulator retries forever.

## Run 1 (0.1.53 merge, before the genesis fix)

rpc-compat 1/35. Every test after "client launch" failed because the node
exited on `ClockError::PreGenesis` (hive writes `GENESIS_TIME` a few seconds
ahead). Fixed: `docs/pq-devnet/wait-for-genesis-2026-09-24.md`.

## Run 2 (0.1.54 + genesis wait)

| suite | result | failures |
|---|---|---|
| rpc-compat | 29 / 35 | the six checkpoint-sync scenarios |
| sync | 1 / 7 | late joiner (helper mesh and same-client), head recovery, bad-checkpoint rejection |
| client-interop | stopped | six nodes with `ethean-prover` exhausted 15 GiB host RAM |

rpc-compat failures: after `--checkpoint-sync-url` the live
`ForkChoiceStore` still held the genesis anchor, so `/lean/v0/fork_choice`
listed the slot-0 node and genesis checkpoints. Fixed the same evening
(`apply_anchor` rebuilds the store from the anchor pair; see
`docs/storage/checkpoint-sync-client-2026-09-24.md`).

sync failures are open gaps, not tonight's work:

- Late joiner: Ethean did not finalize within 300 s of catching up to the
  leanSpec helper mesh (head tracked, finalized slot stayed 0). Needs the
  req/resp block sync from a spec peer plus vote aggregation on the joined
  chain.
- Same-client late joiner: a 3-validator Ethean-only mesh did not finalize
  within 300 s (aggregator + Type-2 merge cost on a shared host).
- Head-behind-finalized recovery: `fork_choice` not readable within 180 s
  during pre-pause catch-up.
- Two scenarios failed inside the helper itself (its head stayed at 0), so
  they say nothing about Ethean yet.

## Run 3 (re-anchor fix, `--sim.limit 'rpc|sync|health'`)

rpc-compat 33 / 35 (the six checkpoint-sync scenarios now pass except two).
The remaining two, `checkpoints justified post-genesis` and `finalized block
pairs with finalized state`, exposed a third bug: right after the checkpoint
anchor the validator tried to publish its first attestation, gossipsub had no
mesh peer yet, and the `InsufficientPeers` publish error propagated out of
the duty step as fatal (`Duty step failed; shutting down`). Fixed the same
evening: a publish with no peers is dropped with a warning and the loop
continues. The sync suite was stopped after rpc-compat because the host had
1 GiB of RAM left; its run-2 results stand.
