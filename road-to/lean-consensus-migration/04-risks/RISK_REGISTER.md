# Lean Consensus Migration Risk Register

## Scoring and closure

Impact is **Critical**, **High**, **Medium**, or **Low**. Likelihood is **Likely**, **Possible**, or **Unlikely**. Critical risks block all network signing or release promotion. High risks block the affected phase. A risk closes only when its acceptance test passes against pinned artifacts and its production metric and alert are live.

| ID | Risk and baseline evidence | Impact / likelihood | Required control and closure evidence |
|---|---|---|---|
| CR-01 | XMSS/WOTS one-time leaf is reused after retry, concurrent duties, restore, clock change, or database rollback. Current signing accepts raw key bytes and persists no leaf state. | Critical / Likely | Transactional reserve-before-sign allocator; monotonic key epoch; signed audit record; restore fencing; concurrency, crash, rollback, and clone tests; zero duplicate `(key_id, leaf_index)` in durable audit and telemetry. |
| CR-02 | A filesystem or VM snapshot restores an older leaf counter while signed messages remain valid on the network. | Critical / Possible | Bind signer state to an external monotonic generation or operator-approved rekey; reject lower generations; checkpoint signing state separately from consensus state; destructive restore drill proves signing remains disabled until reconciled. |
| CR-03 | Simplified/fake cryptography reaches a network build. `poseidon_hash` is currently a transformed SHA-256 result and BLS remains the benchmark path. | Critical / Likely | Compile-time feature separation, production build denial, known-answer vectors, binary/SBOM inspection, and network handshake exposing only the pinned suite. |
| CR-04 | Poseidon parameters, field encoding, domain separation, or round constants differ from leanSpec/leanSig. | Critical / Possible | Pin specification and implementation commits; independent vector agreement with at least two peer clients where available; reject unknown suite IDs; cryptography review signs the pin manifest. |
| CR-05 | leanSig/leanMultisig/prover dependencies drift or an upstream change invalidates proofs/signatures. | Critical / Possible | Exact source and artifact hashes, locked transitive graph, conformance vectors, reproducible build comparison, and controlled pin-update procedure. |
| PR-01 | Prover saturates CPU or RAM, hangs, leaks workers, or misses cancellation, starving consensus. | Critical / Likely | Dedicated bounded worker pool/process, cgroup/job limits, deadline propagation, cooperative plus forcible cancellation, queue rejection, heartbeat/watchdog, and fault tests. |
| PR-02 | Invalid or adversarial witnesses trigger disproportionate proving/verification cost. | High / Possible | Preflight structural validation, input byte and element limits, cost accounting, per-source admission control, bounded retries, and adversarial corpus benchmarks. |
| TM-01 | Block import, duties, signatures, or gossip miss the four-second slot. | Critical / Likely | End-to-end critical-path traces; benchmark-derived stage budgets whose sum plus safety reserve is below four seconds; deadline-aware work shedding; multi-node degraded-network tests. |
| CP-01 | An unauthenticated or stale checkpoint causes long-range capture or inconsistent state. Current checkpoints have zero state roots and no trust metadata. | Critical / Likely | Explicit trust root and provenance policy, canonical state-root verification, weak-subjectivity/finality constraints from the pinned spec, age/network/fork checks, and two-step operator confirmation for trust-root changes. |
| DC-01 | Malformed SSZ or Snappy input causes allocation bombs, decompression bombs, deep decoding, panic, or CPU exhaustion. Current code does not implement canonical SSZ/Snappy. | Critical / Likely | Compressed and decompressed byte caps before allocation, streaming/bounded decode, canonical SSZ checks, nesting/list limits from pinned types, timeout/cost limits, fuzzing, and peer penalties after attributable invalid input. |
| NW-01 | Peer eclipse, churn, mesh collapse, backpressure, duplicate flood, or topic starvation prevents timely consensus. Current queues are unbounded and message IDs can collide within one second. | Critical / Likely | Bounded queues by bytes/items, protocol message IDs, diversity-aware discovery, mesh health rules, overload shedding, reconnect budgets, multi-node partition/churn tests, and liveness SLOs. |
| ST-01 | Crash between related writes exposes a partial block/state/checkpoint/schema or signing transition. Current checkpoint and migration writes are separate. | Critical / Likely | Atomic write batches/transactions, write-ahead intent, fsync semantics, startup recovery, checksums, and crash injection at every write boundary. |
| ST-02 | Data migration irreversibly mutates the only copy or silently accepts incompatible data. | High / Possible | Copy/backup, preflight, immutable migration manifest, shadow validation, atomic activation pointer, read-old/write-new strategy where feasible, and tested rollback boundaries. |
| OB-01 | Missing, stale, or high-cardinality telemetry hides failures or takes down the node. | High / Likely | Exporter lifecycle/readiness contract, bounded labels, scrape self-metrics, cardinality CI tests, recording rules, dashboards, and synthetic alert validation. |
| SC-01 | Compromised dependency, build image, generator, or binary changes consensus behavior. Broad semver requirements increase drift. | Critical / Possible | Lockfile enforcement, exact git pins, checksums, SBOM, provenance, vulnerability/license review, isolated builders, two-build reproducibility check, and signed release artifacts. |
| OP-01 | Operator exposes metrics with secrets or binds an unauthenticated endpoint publicly. | High / Possible | Loopback default, explicit non-loopback opt-in, network policy, optional TLS/auth at proxy or exporter, redaction tests, and startup log without secret values. |
| MX-01 | Ethean agrees with itself but diverges from other Lean clients. | Critical / Possible | Mixed-client testnets, shared vectors, wire captures, differential state roots, finality/head comparison, and peer-specific evidence pinned by commit. |

## Ownership

- **Crypto owner:** CR-01 through CR-05, signing restoration, vector provenance.
- **Prover owner:** PR-01 and PR-02, worker isolation and cancellation.
- **Consensus owner:** TM-01, checkpoint validation, mixed-client state agreement.
- **Networking owner:** DC-01 and NW-01, codec limits, peer liveness.
- **Storage owner:** ST-01 and ST-02, crash consistency and migrations.
- **Operations owner:** OB-01 and OP-01, exporter, dashboards, alerts.
- **Release owner:** SC-01 and final gate evidence.

One person may fill multiple roles in development, but approval for Critical crypto and supply-chain gates requires a second reviewer.

## Evidence ledger requirements

Each risk entry links to a machine-readable CI result and a human-readable review containing:

- Ethean commit, protocol/spec commit, peer-client commits, compiler, target, and dependency lock hash;
- hardware, OS/kernel, filesystem, network profile, validator/load profile, and dataset hash;
- command, configuration, seed, start/end timestamps, raw metrics location, and result;
- observed failure behavior, residual risk, owner, review date, and expiry trigger.

Peer disagreement is not resolved by majority. The pinned normative specification wins; disagreements become interoperability tests and upstream questions.
