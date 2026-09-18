# Security Gates

## Gate model

Every gate returns pass or fail. Waivers are forbidden for signing safety, fake-crypto isolation, checkpoint authenticity, canonical decoding, and atomic XMSS state. Other temporary exceptions require an expiry, named owner, containment, and release-manager approval. CI artifacts must include immutable input pins and raw results.

## G0 — Evidence and pin gate

Pass requires:

- immutable commits for leanSpec, leanSig, leanMultisig, pq-devnet configuration, and every peer used as evidence;
- documented algorithm suite IDs, parameters, serialization rules, domains, and test-vector hashes;
- `Cargo.lock` enforced with no floating git branch or tag;
- pinned Rust toolchain, build container by digest, Prometheus image, Grafana image, dashboard schema, and provisioning configuration;
- a source-to-binary SBOM and license/vulnerability policy result.

Any unknown pin fails all later gates.

## G1 — Cryptographic correctness and isolation

Pass requires known-answer, negative, boundary, and cross-client vectors for hashing, domain separation, XMSS/WOTS signing and verification, aggregation, and proof verification. At least two independent implementations should agree where peers expose the same pinned protocol; leanSpec vectors remain authoritative.

Production artifacts must:

- contain no runtime switch to simplified Poseidon, deterministic test keys, accept-all verifier, BLS compatibility signing, or mock prover;
- reject unknown parameters, suite IDs, domains, malformed encodings, non-canonical values, and incompatible network/fork identifiers;
- expose the active cryptographic pin hash and suite as bounded build information;
- pass binary symbol/string and dependency-feature inspection.

Test crypto is compiled under an explicit test-only feature that cannot coexist with the production network feature. CI proves the forbidden feature combinations fail to build.

## G2 — XMSS state safety

The signing transaction is `reserve durable leaf -> fsync/commit -> construct signature -> record outcome`. A failed or cancelled signing attempt burns the reserved leaf. It never returns the leaf to the pool.

Pass requires:

- a globally unique key identifier and monotonically increasing signing generation;
- atomic compare-and-swap or serialized transaction for leaf allocation;
- concurrent-process exclusion and duplicate-instance detection;
- durable records for reservation, signature digest, duty coordinates, result, and exhaustion;
- crash injection before and after every persistence/signing boundary;
- rollback, copied-database, stale-backup, partial-write, disk-full, and clock-skew tests;
- restore behavior that starts read-only and refuses signing until generation and network history reconcile;
- zero leaf reuse under all tests and an alert on any impossible duplicate observation.

## G3 — Untrusted data and networking

Pass requires fuzzed and bounded SSZ/Snappy ingress:

- reject compressed frames above the pinned protocol limit before decompression;
- cap decompressed bytes, collection lengths, nesting, total allocations, and decode work from pinned type bounds;
- decode canonically and reject trailing bytes or invalid offsets;
- never panic, abort, or allocate without a checked bound on arbitrary input;
- distinguish invalid, duplicate, stale, locally overloaded, and internal-error outcomes for peer scoring;
- bound gossip, validation, sync, and response queues by bytes and items;
- prove one hostile peer cannot starve duty traffic or consensus processing.

Partition, churn, mesh-loss, slowloris, duplicate flood, decompression bomb, and reconnect-storm tests must preserve process health and recover liveness.

## G4 — Checkpoint and persistence

Pass requires:

- checkpoint state root recomputed from canonical state and matched to authenticated metadata;
- network, fork, finalized root, slot/epoch, source, signature/trust-root identity, and import timestamp recorded;
- stale, future, conflicting, cross-network, and unknown-source checkpoints rejected;
- block/state/index/head/finality writes activated atomically;
- migrations use preflight, intent, versioned writes, validation, and atomic activation;
- kill-at-every-boundary tests recover to exactly old or new state, never a mixture;
- corruption, disk-full, permission, and fsync-error tests fail closed.

## G5 — Prover containment

Pass requires a separately metered execution domain with bounded concurrency, queue size, CPU, resident memory, temporary disk, witness bytes, and wall time. Values come from [PERFORMANCE_BUDGETS.md](PERFORMANCE_BUDGETS.md).

Deadline and shutdown cancellation must stop new admissions, signal active jobs, forcibly terminate non-cooperative workers after the measured grace interval, reclaim resources, and leave no accepted-but-untracked job. A hung prover cannot block fork choice, networking, metrics, storage recovery, or process shutdown.

## G6 — Four-second critical path

Pass requires benchmark-derived budgets for every synchronous stage and end-to-end multi-node validation. The measured worst accepted percentile plus declared safety reserve must fit below the four-second protocol slot. No stage may start work it cannot finish or safely cancel by its deadline. Overload behavior sheds non-critical work before duties.

## G7 — Observability and operations

Pass requires:

- exporter startup failure is explicit; readiness is false until required subsystems and metric registration are healthy;
- graceful shutdown stops readiness, drains in-flight scrape responses, and releases the bind;
- loopback is default; non-loopback binding requires explicit configuration and documented auth/network controls;
- no private keys, signatures, witnesses, message bodies, peer IDs, validator IDs, roots, paths, tokens, or unbounded errors appear as labels;
- all recording and alert rules pass `promtool check rules`;
- all dashboards load through provisioning against pinned Prometheus/Grafana versions;
- synthetic faults fire, route, and resolve the expected alert.

## G8 — Release reproducibility

Two clean builders from the pinned source, toolchain, lockfile, and image must produce matching artifacts after normalized supported metadata. Release provenance includes source commit, lock hash, toolchain, builder digest, features, protocol pins, SBOM hash, test evidence, and signatures. A mismatch blocks publication.

## Promotion sequence

The allowed sequence is offline vectors, single-node no-signing, isolated signing, homogeneous multi-node, mixed-client multi-node, adversarial staging, then the targeted devnet. A failure returns the build to the earliest affected gate; later successes do not override it.
