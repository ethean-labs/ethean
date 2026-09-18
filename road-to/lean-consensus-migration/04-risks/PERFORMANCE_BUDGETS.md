# Performance Budgets

## Principle

The only fixed time in this plan is the protocol's four-second slot. Ethean's current benchmarks contain sleeps, simulated errors, synthetic memory samples, a hard-coded throughput value, and BLS-centric work; they are not admissible evidence for Lean budgets. Phase 00 establishes real baselines before numeric gates are approved.

## Phase 00 measurement contract

For every benchmark, record Ethean/spec/dependency pins, release profile and features, hardware and power mode, CPU topology, RAM, storage, OS/kernel, filesystem, network shaping, validator count, peer count, object sizes, corpus hash, seed, warm-up, sample count, and raw output.

Use at least:

- the minimum supported operator host;
- the reference CI host;
- a representative production host;
- single-node microbenchmarks and homogeneous/mixed-client multi-node runs;
- normal, maximum-valid, burst, malformed, partitioned, disk-pressure, CPU-pressure, and prover-hang profiles.

Report distributions, not averages: sample count, p50, p95, p99, p99.9 where statistically supportable, maximum, confidence interval, error count, CPU time, wall time, peak RSS, allocations, I/O, and queue delay.

## Budget derivation

Each approved budget artifact contains `baseline`, `limit`, `rationale`, `scope`, and `expiry trigger`.

- **Latency limit:** an approved factor or additive margin over the measured Phase 00 distribution, constrained by the four-second end-to-end envelope.
- **Capacity limit:** the largest load passing the latency, memory, correctness, and recovery gates on the minimum host, reduced by an approved safety margin.
- **Memory limit:** measured steady-state plus burst and fragmentation margin, remaining below enforced process/container capacity.
- **Queue limit:** Little's-law-informed measured service capacity plus bounded burst tolerance; both item and byte caps are mandatory.
- **Cancellation limit:** observed cooperative and forced-stop distributions plus cleanup verification.

The review must choose and justify factors after seeing raw data. This document deliberately does not preselect unsupported values.

## Four-second critical-path ledger

Phase 00 measures and assigns a deadline to each stage:

1. receive and bounded Snappy framing;
2. SSZ decode and structural validation;
3. state lookup and pre-state preparation;
4. signature/aggregate/proof verification;
5. state transition and fork-choice update;
6. atomic persistence;
7. gossip validation and onward publication;
8. validator duty preparation, leaf reservation, signing/proving, and publication;
9. scheduler, runtime, and network jitter reserve.

For each path, the sum of stage limits plus reserve must be **strictly less than 4 seconds**. The reserve is measured from stressed multi-node runs, not guessed. If proof generation cannot fit its assigned path, it moves out of the synchronous path or the phase does not promote.

## Required benchmark gates

| Area | Workload and measured outputs | Passing rule |
|---|---|---|
| Hash/sign/verify | Pinned Poseidon, leanSig/XMSS leaf reservation, sign, verify, invalid verify, exhaustion | Correct vectors; stage limit met; zero leaf reuse; bounded memory |
| Aggregation/prover | Representative and maximum-valid committee inputs, valid/invalid witnesses, cold/warm runs | Deadline met; bounded CPU/RSS/temp disk; cancellation and cleanup pass |
| Block/attestation | Normal and maximum-valid SSZ objects, batch and burst arrival | Four-second ledger met; no unbounded queue growth |
| Codec abuse | Compression ratios, malformed offsets, large lists, trailing data, fuzz corpus | Rejected within derived work/memory cap; no panic or process-wide starvation |
| Gossip/sync | Peer and validator scales from pinned devnet profile, churn, loss, delay, duplicates | Required topic liveness and catch-up SLO met; bounded queues and bandwidth |
| Storage | Block/state/checkpoint commits, compaction, restart, migration, crash injection | Atomic recovery; persistence stage limit met; disk growth budget met |
| Exporter | Full metric set, concurrent scrapes, slow/disconnected clients | Consensus latency regression within approved measurement noise; bounded scrape memory |
| Mixed client | Same finalized/head/state-root observations across pinned peers | No protocol divergence and propagation ledger met |

## Prover resource policy

Prover admission uses measured per-job CPU time, peak RSS, temporary disk, and queue delay. The runtime enforces:

- finite worker concurrency and queue bytes/items;
- per-job witness/input cap from pinned protocol bounds;
- absolute deadline inherited from the duty;
- cancellation token and worker heartbeat;
- measured cooperative grace followed by process termination;
- no automatic retry after deadline; retry policy is keyed by deterministic error class;
- resource reclamation verified before capacity is returned.

The node reserves measured CPU and memory headroom for consensus, P2P, storage, and telemetry. Prover work is always lower priority than maintaining consensus liveness.

## Regression policy

CI runs stable microbenchmarks and compares distributions with the approved artifact. Nightly or dedicated hosts run noisy system and multi-node tests. A regression fails when it breaches an approved limit, not merely a percentage chosen in CI code. Baselines cannot be updated in the same unreviewed change that regresses them.

Re-baselining is required after protocol pins, compiler/toolchain, cryptographic implementation, prover backend, database engine, serialization, supported hardware, or material topology changes. Historical artifacts remain immutable for audit.
