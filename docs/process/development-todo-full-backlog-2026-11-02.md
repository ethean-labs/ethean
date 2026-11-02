# Development todo — full backlog (2026-11-02)

Working list for the rest of the Hive / pq-devnet path. Update checkboxes in
this file as items land. Do not invent A2 fork digests, A3 bootnodes, or
fixture bodies. Tests and local meshes: `CARGO_BUILD_JOBS=2`. Do not start a
long soak unless asked. Commit only on request; never push unless asked.

Authority: in-repo leanSpec fixture digest in
`spec/fixtures/phase-00/manifest.toml` (generator `0b7d33ec…`), then the live
pq-devnet pin, then this list. Open leanSpec PRs are notes, not pins.

Related: [what-external-backlog-means-2026-09-24.md](./what-external-backlog-means-2026-09-24.md),
[lean-eth-scan](../../bazalinacaklar/lean-eth-scan-2026-11-02.md) (gitignored).

---

## In progress

### 1. Borrow parent state on block import (no full `State` clone) — done 2026-11-02

`pre_state_for_parent` now returns `&State`. Note:
[borrow-parent-state-import-2026-11-02.md](../pq-devnet/borrow-parent-state-import-2026-11-02.md).

Items 2–12 done 2026-11-02 (no soak). Remaining work is wait-for-pin / external.

---

## Next (in-repo, this order)

### 2. Remaining hot-path copies — done 2026-11-02

- Import takes `DecodedBlockGossip` by value and moves the inner `Block` into
  fork choice after Split seed (no extra block clone).
- `insert_proved` borrows bits/proof; gossip still needs one pool copy of the
  Type-1 bytes.
- Split jobs take the key list by move on the last attestation.
- Serve cache is filled once via `put_block_at_slot` (by-root map is the same
  bytes). Own-block publish moves the payload into the cache.
- Follow-up: FC init borrows head state; own-block durable copy happens at
  publish, not while the gossip is still pending.

Note: [hot-path-copies-2026-11-02.md](../pq-devnet/hot-path-copies-2026-11-02.md).

### 3. Type-2 still longer than the 4 s slot — code 2026-11-02; live timing not re-run

Interval-2 Type-1 is skipped when this node proposes N+1 **and** the pool
already has a proved body (`defer_type1_for_early_block`). Empty pool still
proves so a lone aggregator can fill the next block. leanVM cancel is still
absent. Do not start a host soak unless asked.

Note: [yield-type1-to-early-block-2026-11-02.md](../pq-devnet/yield-type1-to-early-block-2026-11-02.md).

Remaining: Type-2 itself is still ~3.3 s for two components (spec/prover).

### 4. Two-aggregator finality on one host — code 2026-11-02; soak not re-run

Union stays off the proposer path. At most one union (and no union while a
Type-1 is in flight) per process. Note:
[cap-union-jobs-2026-11-02.md](../pq-devnet/cap-union-jobs-2026-11-02.md).
Re-run the 2-aggregator layout only when asked.

### 5. Checkpoint-sync joiner polish — docs 2026-11-02

`--checkpoint-sync-url` fetches the finalized pair at start. `usage.md` and
the Hive CLI table now say so. `https://` is refused. Empty-body 3SF lag
after a late join is expected (see
[late-join-restart-mesh-2026-09-29.md](../pq-devnet/late-join-restart-mesh-2026-09-29.md)).
Do not start a joiner soak unless asked.

### 6. Health that is not just process liveness — done 2026-11-02

`/readyz` fails on a frozen head, a dead prover worker, or a Type-2 stuck
for 8 slots. Gossip idle is a gauge only. Note:
[progress-health-2026-11-02.md](../observability/progress-health-2026-11-02.md).

### 7. Bound remaining untrusted queues — done 2026-11-02

`known_payloads` and the Type-1 pool share the 256-key cap; all three pools
prune on every tick. `block_states` still only drops finalized history (spec).
Note: [bound-untrusted-queues-2026-11-02.md](../pq-devnet/bound-untrusted-queues-2026-11-02.md).

### 8. Fork-choice tests the peers still trip on — done 2026-11-02

Unit tests for fresh vs activated votes, unknown-head pending votes, tick in
the past, prune of stale votes, duplicate proposal, and deferred replay after
import. Fixture digest unchanged. Note:
[fork-choice-peer-trip-tests-2026-11-02.md](../pq-devnet/fork-choice-peer-trip-tests-2026-11-02.md).

### 9. `just apitest` / Hive HTTP gaps — done 2026-11-02

`/events` is JSON/SSE (not 501). Hive rpc_compat five paths unchanged.
`/lean/v0/chain/{head,finalized,sync}` aliases the v1 handlers. Note:
[apitest-http-gaps-2026-11-02.md](../hive-testing/apitest-http-gaps-2026-11-02.md).

### 10. Gossip fork digest vs SHA-256(`fork_name`) — done 2026-11-02

Default remains lstar `GOSSIP_DIGEST` `12345678`, not SHA-256(`fork_name`).
Bootnode dials without `--fork-digest` still warn. No A2 value invented.
Note: [fork-digest-warn-only-2026-11-02.md](../networking/fork-digest-warn-only-2026-11-02.md).

### 11. Scale: more nodes / validators / subnets — flags only 2026-11-02

`local-devnet.ps1` already has `-ValidatorsPerNode` / `-Subnets` / `-Aggregators`.
No larger run on this CPU. Note:
[scale-flags-no-soak-2026-11-02.md](../pq-devnet/scale-flags-no-soak-2026-11-02.md).

### 12. Docs honesty pass — done 2026-11-02

`roadmap.md` no longer lists QUIC/RocksDB/FFI as open compile gates. The
v0.1.70 FC backlog page is labeled a snapshot. Note:
[docs-honesty-2026-11-02.md](./docs-honesty-2026-11-02.md).

---

## Wait for a spec / operator pin (do not implement as if merged)

| Item | Why it waits |
| --- | --- |
| Goldfish / PQ heartbeat | leanroadmap pq-devnet-5 **Planned**; still 3SF-mini |
| leanSpec #1207 (vote justified if target walk is behind) | Open PR; we skip the vote today |
| leanSpec #1208 (keep pending proofs a round skips) | Needs merged spec diff |
| Fixture digest bump to Gean `eca701ef` (or GitHub `latest`) | In-repo sha256 wins until we re-import |
| Engine API / execution payload | Future devnet; Ethean is consensus-only |
| Gossipsub v2 / set reconciliation / 1 ETH validator P2P | Research track, no wire pin |
| APS / rainbow staking | Research track |
| leanSPHINCS / Flock proving line | Cambridge 2026 talks; not the leanVM pin |
| A2 fork digest + A3 bootnodes | Operator paste; empty config files on purpose |

leanSpec #748 (skip trivial Type-1) is **already in the tree** this session.

---

## External (not owned by this git tree)

| Item | Blocker |
| --- | --- |
| ethereum/hive `clients/ethean` PR | Other org + `gh auth` |
| leanstart Kind/Helm Ethean image row | Docker / GHCR public |
| Hive live `:devnet5` smoke | Docker Desktop on this host (do not change WSL/Hyper-V) |
| leanSpec `fill` for empty-body FC weights | Upstream fixture archive |
| Public pq-devnet join | A2/A3 paste |

---

## Out of scope (do not start)

- Beacon `/eth/v1`, BLS, 12 s slots, execution client
- Inventing bootnodes or fork digests
- Cloning peer clients into this repo
- Auto-push, Cursor git trailers, version bump except on requested commit
- Changing Windows power / WSL / Hyper-V settings

---

## Already done (do not redo)

Session work through 2026-11-02, still **uncommitted** unless the operator
asked to commit: early two-interval block proof, copy cuts, aggregation
head-state borrow, serve-only-verified sync blobs, Type-1 yield to in-flight
block, trivial Type-1 skip, libp2p 0.57 / redb 4.3 notes in Unreleased.

Pinned product: workspace **0.1.72**, leanVM **e2592df4**, default
`--max-block-attestation-data` **1**.
