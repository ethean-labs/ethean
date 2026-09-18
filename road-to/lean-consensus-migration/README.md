# Ethean Lean Consensus Migration

This directory is the planning authority for replacing the current Beacon-era prototype with the **Ethean Lean Consensus Client**. It defines a full replacement migration. Existing implementation code is evidence of responsibilities and failure modes, not a base that receives retain status.

## Decision

- Product name: **Ethean Lean Consensus Client**.
- Protocol authority: `leanSpec` `main`, frozen to an exact commit and fixture artifact digest for each phase.
- Cryptography, networking, metrics, quickstart, and devnet inputs are frozen alongside the phase's `leanSpec` pin.
- Every old source surface ends as **replace-then-delete**, **rename-by-replacement**, or **delete**.
- No existing source file, protocol type, wire format, persistence record, test, benchmark, example, deployment artifact, or protocol claim has retain status.
- Generic ideas may be reimplemented after review, but copying an old implementation does not satisfy a replacement gate.
- Unknown protocol values are Phase 00 blockers. They are never filled with peer-majority values or legacy Beacon defaults.

## Documents

### Charter

- [Project charter](./00-charter/PROJECT_CHARTER.md)
- [Scope and non-goals](./00-charter/SCOPE_AND_NON_GOALS.md)
- [Success criteria](./00-charter/SUCCESS_CRITERIA.md)
- [Charter index](./00-charter/README.md)

### Baseline

- [Current-state audit](./01-baseline/CURRENT_STATE_AUDIT.md)
- [Legacy component matrix](./01-baseline/LEGACY_COMPONENT_MATRIX.md)
- [Removal, rewrite, and reuse map](./01-baseline/REMOVAL_REWRITE_REUSE_MAP.md)
- [Baseline index](./01-baseline/README.md)

### Protocol

- [Authority policy](./02-protocol/AUTHORITY_POLICY.md)
- [Compatibility ledger](./02-protocol/COMPATIBILITY_LEDGER.md)
- [Upstream refresh policy](./02-protocol/UPSTREAM_REFRESH_POLICY.md)
- [Open spec decisions](./02-protocol/OPEN_SPEC_DECISIONS.md)
- [Protocol index](./02-protocol/README.md)

### Architecture

- [Target workspace](./03-architecture/TARGET_WORKSPACE.md)
- [Dependency rules](./03-architecture/DEPENDENCY_RULES.md)
- [Data flow](./03-architecture/DATA_FLOW.md)
- [Module size policy](./03-architecture/MODULE_SIZE_POLICY.md)
- [Architecture index](./03-architecture/README.md)

### Risks

- [Risk register](./04-risks/RISK_REGISTER.md)
- [Security gates](./04-risks/SECURITY_GATES.md)
- [Performance budgets](./04-risks/PERFORMANCE_BUDGETS.md)
- [Data migration policy](./04-risks/DATA_MIGRATION_POLICY.md)
- [Risks index](./04-risks/README.md)

### Retirement

- [Deletion register](./05-retirement/DELETION_REGISTER.md)
- [Required directory policy](./05-retirement/REQUIRED_DIRECTORY_POLICY.md)
- [Legacy name allowlist](./05-retirement/LEGACY_NAME_ALLOWLIST.md)
- [Retirement index](./05-retirement/README.md)

### Observability

- [Metrics contract](./06-observability/METRICS_CONTRACT.md)
- [Prometheus topology](./06-observability/PROMETHEUS_TOPOLOGY.md)
- [Grafana dashboards](./06-observability/GRAFANA_DASHBOARDS.md)
- [Alerting and SLOs](./06-observability/ALERTING_AND_SLOS.md)
- [Multinode analysis](./06-observability/MULTINODE_ANALYSIS.md)
- [Observability index](./06-observability/README.md)

### Phases

- [Phase index (00–13)](./phases/README.md)
- Critical path: `00 → 01 → 02 → 03 → 04 → 05 → 06`, then signer/aggregation/duties (`07–09`), network/storage/sync (`10–11`), interop/release (`12–13`).
- Charter “owner phases” group these detailed plans; implementation work tracks the numbered phase files.

## Authority and evidence

Authority is applied in this order:

1. the phase-frozen `leanSpec` commit and verified fixture bundle;
2. matching `leanSig`, `leanVM` or `leanMultisig`, pq-devnet profile, networking schema, and metrics pins;
3. current pq-devnet interoperability results;
4. peer clients at the exact locally audited commits;
5. the existing Ethean tree as legacy evidence only.

The local peer library was audited at these snapshots:

- Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`
- Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`
- Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`
- ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`
- Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`
- Gean `b78f6d737f4df57a72d5e230635681235fda8024`
- Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`

Peers agree on broad responsibilities—bounded SSZ, hash-tree roots, slot-oriented state transition, separate proposal and attestation XMSS keys, recursive proof aggregation, QUIC/Gossipsub, SSZ/Snappy, status and block recovery—but disagree on protocol generation and exact values. Peer agreement is not a pin.

## Replacement phases

Detailed executable plans live under [phases/](./phases/README.md) (files `00`–`13`). The charter groups those plans into owner phases for accountability; do not treat the grouped labels as a second numbering scheme.

| Group | Detailed phases | Outcome |
| --- | --- | --- |
| Protocol and security freeze | 00 | Immutable compatibility ledger; unresolved values remain blockers |
| Product skeleton | 01–02 | Ethean identity and clean workspace; legacy package identity removed |
| Types and consensus core | 03–06 | SSZ/types, genesis/clock, state transition, fork choice/finality |
| Signer, aggregation, duties | 07–09 | XMSS safety, leanVM proofs, validator/node scheduling |
| Network, storage, sync | 10–11 | QUIC/gossip/req-resp, atomic storage, checkpoint trust |
| Operations and release | 12–13 | Lean API, Prometheus/Grafana, mixed-client interop, release gates |

Phase 12 exit requires at least two Ethean nodes and one peer client scraped as distinct Prometheus targets, auto-provisioned Grafana dashboards, and synthetic finality-stall / prover-timeout / peer-loss alerts.

## Phase rules

- A phase pin is immutable after implementation begins. Upstream movement creates a new phase pin and explicit migration, not silent drift.
- `Cargo.lock`, fixture digests, generated-code provenance, and binary version output must identify the same compatibility ledger.
- Unsupported fixture types, skipped mandatory vectors, mocked production verification, mutable `latest` downloads, and branch-only crypto references fail the gate.
- Production validation fails closed on profile mismatch.
- All attacker-controlled collections, decompression, proofs, queues, streams, caches, and pending graphs are bounded before expensive work.
- Stateful signing is atomic, role-separated, crash-safe, rollback-aware, and tested before validator duties are enabled.
- Source files remain at most 300 lines and are split by responsibility.
- All repository prose, identifiers, diagnostics, and operator-facing text are English.

## Planning-boundary warning

This library is planning authority only. Accepting these documents does not by itself change product `src/` code. Implementation begins only when a numbered phase plan is executed and its entry criteria are met. Legacy Panro/Beacon roadmap files under `road-to/` were removed after facts were copied into baseline and retirement documents; Git history remains the archive.
