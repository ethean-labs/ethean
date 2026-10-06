# Larger local devnet: several validators per node, several aggregators (2026-09-30)

Follow-up to [local-proving-devnet-windows-2026-09-29.md](./local-proving-devnet-windows-2026-09-29.md).
Goal: run the proving mesh with more validators than nodes and with one
aggregator per attestation subnet, the shape pq-devnets use.

## What changed

### Several validators per node

`load_node_keys` kept one proposal and one attestation key per node, and the
attest duty signed only `owned_validator_indices.first()`. A node owning
indices 1, 3, 5 therefore signed every vote as validator 1 with whichever key
was read last, and all its votes failed verification.

- `registry_keys.rs`: `LoadedNodeKeys` now holds one `ValidatorKeys`
  (proposal + attestation record) per index; a second key of the same role for
  one index is an error.
- `registry_apply.rs`: every validator's keys are installed into
  `ChainOwner::signers` (`BTreeMap<u64, ValidatorSigners>`); each XMSS secret is
  tracked for background tree preparation, and the prover starts when any
  proposer key was installed.
- `duty_attest.rs`: the attestation data is computed once, then each owned
  validator with an attester signs, verifies, pools (when this node aggregates
  that subnet) and gossips on its own subnet `index % committees`.
- `duty_propose.rs`: the block is signed with the key of
  `block.proposer_index`, not with a node-wide proposer.
- The single `attester` / `proposer` fields remain for solo and smoke runs and
  are ignored once registry signers are installed.

### Subnet-scoped aggregation

leanSpec aggregators subscribe to their validators' subnets plus
`aggregate_subnet_ids`, and aggregate only what they receive there.

- `ChainOwner::aggregate_subnet_ids` comes from `--aggregate-subnet-ids`.
  `aggregates_vote_of(index)` / `aggregates_subnet(s)` decide what an aggregator
  pools. With no owned indices and no ids it pools everything (old behavior).
- `gossip_attestation.rs` pools a gossip vote only when this aggregator covers
  the voter's subnet.

### Union merge across subnets

With two subnets each aggregator proves at most half of the votes, so no single
pooled proof reaches 2/3. leanSpec folds such proofs into a child merge while
building the block (`block_production.py`). Ethean keeps that work off the
proposer's path:

- `AggregatePool::mergeable_keys_since(min_slot)` lists data with at least two
  proved variants, one of them recent.
- `aggregation_duty::schedule_union_merges` merges those variants (greedy by new
  coverage, at most `MAX_CHILDREN`) with no raw signatures, for data of the last
  `UNION_WINDOW_SLOTS = 2` slots. Only the aggregator that owns subnet
  `data.slot % committees` runs it, so the union is proved once.
- Merges are checked on every interval (`schedule_union_merges_only`), not only
  at the once-per-slot aggregation interval, so a partial aggregate that lands
  late is merged without waiting a full slot. A finished union is the widest
  variant, so the same data is not merged twice.
- Debug logs: `union merge queued`, `aggregate proved` (slot, coverage, time).

### Devnet tooling

- `ethean devnet-init --attestation-committee-count C --aggregators A`, with
  `--validators-per-node` as before. `config.yaml` gets
  `ATTESTATION_COMMITTEE_COUNT: C`; nodes `0..A` print `--is-aggregator`.
- Assignment is subnet-homogeneous: node `k` owns indices
  `s + C * (g * vpn + v)` with `s = k % C`, `g = k / C`, `v < vpn`. Each node's
  validators sit on one subnet and aggregators `0..C` cover one subnet each. With
  one validator per node this is the identity. With more, `nodes` must be a
  multiple of `C` (refused otherwise).
- Keys are generated per index and reused across runs, whatever the layout.
- `scripts/local-devnet.ps1 -ValidatorsPerNode n -Subnets C -Aggregators A`.

## Runs (one Windows host, 20 cores, leanVM `e2592df4`, 300 s)

| Layout | Justified | Finalized | Notes |
| --- | --- | --- | --- |
| 4 nodes × 3 validators, 1 subnet, 1 aggregator | 1 → 3 → 9 → 20 → 25 → 32 → 45 → 56 → 62 → 65 | 20 → 56 → 62 | heads agree; finalized trails head by about 13 slots |
| 4 × 3, 2 subnets, 2 aggregators (before per-interval merges) | 2 → 4 → 25 | 0 | union merges rare, aggregates cover 3 or 6 |
| 4 × 3, 2 subnets, 2 aggregators | 2 → 16 → 36 | 0 | 8 union merges, 5 full (12/12) aggregates |
| 6 × 1, 2 subnets, 2 aggregators (earlier, one validator per node) | slow | 0 | merges 5–6.7 s against 4 s slots, forks |

Before the key fix the 4 × 3 layout stayed at justified 0 with invalid vote
signatures.

The single-subnet control shows that several validators per node now work end
to end. The two-subnet runs justify but do not finalize within 300 s:

- A union merge costs about 4.9 s on average (max 7 s), as much as a block
  proof, because it is a recursive child merge. A Type-1 over raw votes takes
  about 1.3 s.
- In the second two-subnet run, 82 of 144 aggregates covered only 3 validators.
  The two nodes of a subnet had voted for different heads, so their votes have
  different data roots and cannot be merged. Heads differ at vote time because
  block proofs of 5–7 s push blocks one to three slots late.
- Every node proves its own blocks and both aggregators also prove Type-1 and
  union jobs on the same CPU. The leanVM `parallel` crate sizes its pool from
  `available_parallelism()` with no knob, so provers oversubscribe the host.

So one host is too small for a multi-aggregator mesh at this prover pin. The
code paths work (full 12/12 unions were built and imported), but finality with
split subnets needs separate machines or a faster prover. The default stays at
1 subnet and 1 aggregator.

## Commands

```powershell
.\scripts\local-devnet.ps1 -Nodes 4 -ValidatorsPerNode 3 -RunSeconds 300 -DebugLog
.\scripts\local-devnet.ps1 -Nodes 4 -ValidatorsPerNode 3 -Subnets 2 -Aggregators 2 -RunSeconds 300 -DebugLog
```

The first run with a new `-Out` generates two XMSS keys per validator (about 20 s
each); later runs reuse them.

## Follow-ups

- Multi-host run (or leanstart / Hive once Docker works) for the two-subnet
  layout.
- Several provers now share the cores. See
  [prover-thread-share-2026-09-30.md](./prover-thread-share-2026-09-30.md):
  `ETHEAN_PROVER_COUNT` gives each prover an equal share (at least 8 threads),
  and with it the 4 × 3, 2-aggregator layout finalized for the first time.
- The few `no pending status for peer` lines at startup were repeated Status
  requests; fixed in
  [status-request-dedupe-2026-09-30.md](../networking/status-request-dedupe-2026-09-30.md).
