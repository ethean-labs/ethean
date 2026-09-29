# Safe-target vote merge and block data cap 1 (2026-09-29)

Follow-up to [mesh-finality-fixes-2026-09-29.md](./mesh-finality-fixes-2026-09-29.md).
With those fixes the 3-node local proving devnet
([local-proving-devnet-windows-2026-09-29.md](../pq-devnet/local-proving-devnet-windows-2026-09-29.md))
finalized once, then kept justifying without finalizing (justified
3 → 6 → 25 → 49 → 64 in 300 s, finalized stuck). Three causes, plus some
cleanup.

## 1. Non-aggregators computed the safe target from stale votes

leanSpec `update_safe_target` reads only `latest_new_aggregated_payloads`.
Single gossip votes carry no fork-choice weight until an aggregate that covers
them arrives. On a non-aggregator node the aggregate for slot N often lands
after its own and its peers' slot N+1 single votes, so the safe target ran one
or two slots behind the aggregator's. Different safe targets gave different
`get_attestation_target` results, and votes split across targets.

`relevant_new_votes` (`crates/fork-choice/src/head/weights.rs`) now merges
pending payload votes with `latest_new_attestations` and keeps the newer slot
per validator. This is a documented deviation from the spec. Test:
`late_aggregate_does_not_hide_newer_single_votes`. After the change the
`s=` column of the devnet poll matched on all nodes.

## 2. Block data cap default 3 → 1

A block proof is the proposer's singleton Type-1 aggregate plus a Type-2 merge
with one component per attestation data plus the proposer's
([prover-timing-block-data-cap-2026-09-29.md](../lean-crypto/prover-timing-block-data-cap-2026-09-29.md)).
With cap 3 the merge often reached four components (~5.5 s). Once the proof
takes longer than the 4 s slot, the next proposer builds on the grandparent,
the chain forks, `get_attestation_target` falls back to the justified
checkpoint, and votes with target == source count for nothing.

Measured on the same host, 300 s each:

| Cap | Merge avg per node | Merge max | Finality |
| --- | --- | --- | --- |
| 3 | 3.0 / 4.7 / 4.4 s | 13.8 s | none after the first |
| 1 | 2.5 / 3.5 / 3.8 s | 6.3 s | finalized 49, then 56 |

Once the mesh agrees on the target, one data carries almost every vote, so the
concern that split votes would not reach 2/3 did not show up.
`DEFAULT_MAX_BLOCK_ATTESTATION_DATA` is now 1; `--max-block-attestation-data`
still accepts 0–8.

Under a cap, `select_body_capped` orders candidates with the same target slot
by best proof coverage (widest first), so the one slot goes to the data with
the most votes rather than the lowest data root. At the spec maximum the
order is unchanged (target slot, then data root). Test:
`capped_selection_takes_the_widest_data_for_a_target`.

## 3. Aggregation once per slot (leanSpec interval 2)

Two more 300 s runs at cap 1 disagreed: one stalled at justified 20, one
finalized 49 and then 56. New debug lines (`own block proved and imported`,
`gossip block imported` / `skipped`, with parent slot and `is_head`) and a
fork-choice JSON dump per node at the end of the script showed the cause.
Block proofs took 2.8–4.3 s from planning to import with no data and 4.7–8.3 s
with one data. Whenever a proof ran past the 4 s slot, the next proposer
(planning 3.2 s into the slot) had not seen it yet and built a sibling, so
exactly the blocks carrying votes were orphaned.

The prover merge times were 2–7.5 s against 1.7–3.3 s measured solo. The
aggregator had run 165 Type-1 jobs in 300 s (1.24 s each), because
`apply_wall_step` called `schedule_aggregations` on every interval and each
late signature started a new job. On one host that load slowed every node's
block proof. leanSpec lstar `timeline.py::tick_interval` aggregates once per
slot, at interval 2.

`schedule_aggregations` now runs only at
`aggregation_duty::AGGREGATION_INTERVAL` (2). Signatures that arrive later wait
for the next slot's interval 2. Nothing starts in interval 3 any more, so the
earlier "keep the prover free before the early build" gate
(`holds_prover_for_next_block`) was removed.

Result (same host, 3 nodes, cap 1, 300 s):

| Metric | Every interval | Interval 2 only |
| --- | --- | --- |
| Aggregator Type-1 jobs | 165, avg 1.24 s | 105, avg 0.67 s (25 are its block singletons) |
| Merge avg / max per node | 2.5–3.5 s / 7.5 s | 2.4–2.9 s / 3.8 s |
| Finality | none, or 49 → 56 | 3 → 7 → 12 → … → 62 → 67 |
| Head − finalized at the end | 75 − 0 or 75 − 56 | 74 − 67 |
| Blocks proved / stale | — | 72 / 1 (slot 5, at startup) |

## 4. Cleanup

- `try_plan_proposal` does not re-plan while a block proof is still in flight
  on a mesh node (debug log only); the finished proof goes through
  `proof_pre_state` as before.
- `fc_on_tick` returns early when the target time is not ahead of the store
  time, which removed repeated no-op ticks from the debug log.
- `scripts/local-devnet.ps1 -DebugLog` adds `-v`; the poll line prints head,
  safe target, justified and finalized slots (`h= s= j= f=`), and the script
  saves `logs/fork_choice_<k>.json` for every node before stopping them.

## Remaining limit

At leanVM `e2592df4` a block with one data still needs 3–4 s from planning to
import, so blocks land after the interval-1 votes of their own slot and the
margin before the next proposer plans is under a second. One data per block
and spec-timed aggregation keep the chain linear on this host; more data per
block or a slower host would bring the siblings back.
