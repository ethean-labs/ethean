# Provers share the cores (2026-09-30)

Follows [scale-multi-aggregator-2026-09-30.md](./scale-multi-aggregator-2026-09-30.md).

## What the timings showed

`proof_service` logged when a job started but not how long it took, so the
finished-job line now carries `elapsed_ms`. A 4-node devnet (3 validators
each, one aggregator) on this 10-core / 20-thread machine:

| Job | Median | Max |
| --- | --- | --- |
| Block proof, no attestation data | 2.2–2.8 s | 8.2 s |
| Block proof, one attestation | 3.6–4.1 s | 10.1 s |
| Type-1 aggregate (the aggregator node) | 1.1 s | 1.5 s |

A block with one attestation already sits on the 4 s slot. The same layout
with 2 subnets and 2 aggregators never finalized (head 59, finalized 0) and
the proofs roughly doubled:

| Job | 1 aggregator | 2 aggregators |
| --- | --- | --- |
| Block proof, aggregator nodes | 3.6–4.1 s | 4.3–4.4 s (max 11.0 s) |
| Block proof, other nodes | 2.2–2.8 s | 7.4–10.0 s (max 23.5 s) |
| Type-1 aggregate | 1.1 s | 1.4–1.5 s (max 10.2 s) |

The prover process itself never changed. What changed is how many of them run
at once: each `ethean-prover` sizes its pool from `available_parallelism()`
with no limit, so 4 provers put 80 threads on 20 cores. The block proof, which
parallelises well, slowed the most.

## The share

`ethean-prover` reads `ETHEAN_PROVER_COUNT`. When it is above 1 and
`LEANVM_NUM_THREADS` is not already set, it caps its own pool at the cores
divided by that count, never below 8. The node forwards the variable when it
spawns the prover (`ProverConfig::prover_count`), and `local-devnet.ps1` sets
it to the node count. A single prover is left uncapped.

`LEANVM_NUM_THREADS` is honoured in `vendor/leanvm-windows/system-info`, which
is where leanVM asks for the thread count. `0` or an unparseable value leaves
the pool uncapped.

8 is the floor because the proof loses more from a small pool than the host
loses from moderate oversubscription. On this machine, 4 nodes, 2 aggregators,
240 s:

| Threads per prover | Block proof median | Finalized at head ~58 |
| --- | --- | --- |
| 20 (uncapped, 80 threads total) | 4.3–10.0 s | 0 |
| 4 (20 threads total) | 5.7–11.6 s | 0 |
| 5 (cores / nodes) | 5.0–7.6 s | 0 |
| 8 (32 threads total) | 4.0–9.4 s | 36 |

The 8-thread share is the only one that finalized. The single-aggregator
layout still finalizes with it: head 42, finalized 36 after 180 s, block
proofs 3.0–5.1 s.

## Not fixed here

Even with the share, a block proof median of 4–9 s means blocks land late and
the two subnets often vote for different heads, so their aggregates cannot
merge. Finality with 2 aggregators on one host is now possible but still slow.
The next gain has to come from the proof itself (fewer components, or a faster
prover), not from scheduling.
