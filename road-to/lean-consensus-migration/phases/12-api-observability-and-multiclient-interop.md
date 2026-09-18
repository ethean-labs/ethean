# Phase 12 — API, Observability, and Multiclient Interop

## Pinned inputs

- Snapshot `LC-D5-2026-09-19`; Ethean baseline `880982f`; `leanSpec` `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; Phase 00 fixture SHA-256 manifest.
- Observability contract: [../06-observability/README.md](../06-observability/README.md), [METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md), and [PROMETHEUS_TOPOLOGY.md](../06-observability/PROMETHEUS_TOPOLOGY.md).
- Prometheus and Grafana image digests, dashboard schema version, provisioning paths, and alert rule bundles are frozen in `spec/pins/phase-12.lock.toml`.
- Rust `1.97.1`; lockfile, exporter bind defaults, metrics schema version, and build image are immutable within the phase.
- Interop peers: Ream `b003b250f51c038cd5e16b8da02694ee0db1997e`, Zeam `6495beb6b1a584e41c12b3569d9a509abd906259`, ethlambda `313b22d4aa15174b87319002d813926ab7eb6411`, Lantern `440e34727ab3fb447de68d91a1e49be5327706e8`, gean `b78f6d737f4df57a72d5e230635681235fda8024`, and Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`.
- Hive or equivalent mixed-client harness pin and devnet profile from Phase 00.

## Objective

Expose a narrow Lean HTTP/WebSocket API and secure admin/event surfaces, ship a bounded Prometheus exporter with auto-provisioned Grafana dashboards and alert/SLO rules, and prove homogeneous and mixed-client interop with distinct fleet observability.

## Non-goals

- No Beacon REST compatibility, `/eth/v1/` routes, validator-deposit semantics, execution-layer JSON-RPC, or unauthenticated admin on non-loopback bind.
- No ad-hoc metric names, high-cardinality labels, or dashboards that query node internals directly.
- No claim of production readiness without passing the observability and mixed-client exit gates below.
- Peer metric naming is mapped for fleet analysis only; Ethean does not copy peer namespaces blindly.

## Entry criteria

- Phases 03–11 pass; node command loop, sync gate, signer safety, network retrieval, and durable storage are stable.
- [../06-observability/](../06-observability/) documents are populated with Phase 00 evidence and cardinality budgets.
- Deployment manifests define scrape topology, TLS/auth policy, Grafana provisioning, and Alertmanager routing placeholders.
- At least one pinned peer client can join the same devnet profile for mixed-client tests.

## Exact old and new paths

Replace then delete:

- `src/api/` including Beacon validator, beacon, node, and debug routes.
- In-process-only statistics and synthetic performance counters wired into production paths without exporter lifecycle.
- Legacy HTTP middleware, CORS defaults, and unauthenticated debug handlers in `src/client.rs`.

Create:

- `crates/ethean-rpc/src/{lib.rs,server.rs,routes.rs,admin.rs,events.rs,auth.rs,limits.rs}`.
- `crates/ethean-rpc/src/routes/{node.rs,chain.rs,validator.rs,health.rs}`.
- `crates/ethean-metrics/src/{lib.rs,registry.rs,record.rs,export.rs,readiness.rs}`.
- `crates/ethean-metrics/src/families/{chain.rs,network.rs,storage.rs,validator.rs,prover.rs}`.
- `deploy/observability/{prometheus/,grafana/provisioning/dashboards/,grafana/provisioning/datasources/,alerting/rules/}`.
- `tests/interop/hive/`, `tests/interop/mixed_client.rs`, `tests/observability/{exporter,alerts,dashboards}.rs`, and `tests/security/rpc_auth.rs`.

New code directories require English `README.md` files; all hand-written source files are at most 300 lines.

## Ordered tasks

1. Define the Lean API surface: head/finalized/checkpoints, sync status, node health, bounded validator duty visibility, and explicit checkpoint trust metadata. Reject unknown routes and oversize bodies.
2. Implement secure admin and event streams with authentication, rate limits, bounded backlog, and redaction. Separate public health from privileged control.
3. Implement the metrics registry per [METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md): closed label enums, schema version, build info, readiness gauges, and slot-phase histograms.
4. Start the Prometheus exporter with fatal bind failure, `/metrics`, `/healthz`, `/readyz`, graceful shutdown, and readiness false until storage, crypto, signer, network, and prover gates pass.
5. Provision multi-dashboard Grafana bundles: node overview, consensus/finality, network/gossip, storage/sync, validator/duties, prover containment, and mixed-client fleet comparison.
6. Author alert rules and SLO recording rules for finality stall, prover timeout, peer loss, exporter down, readiness false, checkpoint trust failure, and signer unsafe state. Lint with pinned `promtool`.
7. Wire RPC and metrics to typed node/sync/validator handles only; never read databases or mutable chain state directly from HTTP handlers.
8. Run homogeneous multi-node tests with distinct scrape targets and verify dashboard panels populate from Prometheus only.
9. Run mixed-client interop: at least two Ethean nodes plus one pinned peer on the same profile; compare heads, finality, gossip participation, and scrape labels distinctly per client class.
10. Run Hive or equivalent harness cases for startup, sync, finality progression, and graceful shutdown across client combinations.
11. Inject synthetic faults: finality stall, prover hang, peer partition/loss, exporter slow scrape, and clock skew; confirm alerts fire, route, and resolve.
12. Prove telemetry stays inside Phase 00 CPU, memory, and scrape-latency budgets during duty load.

## Deletion obligations

- Delete Beacon API routes, `/eth/v1/` handlers, legacy debug endpoints, and placeholder validator REST templates.
- Remove synthetic counters presented as production telemetry and any route that exposes raw keys, signatures, or unredacted peer identifiers.
- No dual HTTP stack or feature-flagged Beacon compatibility remains.
- Repository scans may mention deleted paths only in migration/deletion records.

## Security/spec risks

- Unauthenticated admin or metrics on non-loopback interfaces leak topology and enable denial-of-service via expensive queries.
- High-cardinality labels (roots, peer IDs, validator indices) can exhaust Prometheus and Grafana.
- Readiness true before signer/network/storage gates pass hides unsafe liveness.
- Mixed-client dashboards can mis-compare incompatible metric semantics if peer evidence is stale.
- Event streams can exfiltrate duty timing or checkpoint trust details if redaction is incomplete.

## Positive and negative fixtures

- Positive: allowed Lean API request/response samples, metrics snapshot with bounded labels, provisioning smoke inputs, and alert rule unit-test series for stall/timeout/peer-loss scenarios.
- Negative: unknown routes, oversize payloads, missing auth, forbidden labels, duplicate metric registration, readiness flip before subsystem ready, and dashboard queries referencing forbidden dimensions.
- Fixture provenance and hashes live in `tests/fixtures/observability/manifest.toml` and the Phase 00 SHA-256 manifest.

## Interop and differential tests

- Homogeneous: two or more Ethean nodes scrape with distinct `instance` labels; Grafana dashboards auto-load and show diverging then converging head/finality during catch-up.
- Mixed-client: two Ethean nodes plus one pinned peer; each client class appears distinctly in fleet panels and alert selectors.
- Hive/mixed-client harness cases for startup, sync, block/attestation propagation, and shutdown.
- Differential metric mapping records peer-only signals for diagnostics; leanMetrics and `ethean_` names remain authoritative for Ethean.

## Validation commands

```text
cargo +1.97.1 test -p ethean-rpc -p ethean-metrics --locked
cargo +1.97.1 test --test exporter --test alerts --test dashboards --locked
cargo +1.97.1 test --test mixed_client --release --locked --features interop
cargo +1.97.1 test --test rpc_auth --locked
promtool check config deploy/observability/prometheus/prometheus.yml
promtool check rules deploy/observability/alerting/rules/*.yml
cargo +1.97.1 clippy -p ethean-rpc -p ethean-metrics --all-targets --locked -- -D warnings
rg -n "/eth/v1/|beacon/api|BeaconApi" crates src deploy
```

## Exit criteria

- Lean API and admin surfaces pass auth, limit, and redaction tests; Beacon routes are absent.
- Exporter lifecycle, readiness gating, and shutdown behavior match [../06-observability/README.md](../06-observability/README.md).
- Grafana dashboards and datasources provision automatically from repo paths without manual UI steps.
- Fleet gate: at least two Ethean nodes and one peer client scrape distinctly; mixed-client panels and alerts differentiate client class.
- Synthetic finality stall, prover timeout, and peer loss alerts fire, route, and resolve in the pinned alert test harness.
- Telemetry remains within approved budgets; touched source files are within 300 lines.

## Rollback/data policy

API and observability configuration rollback is code/config only; metrics history in Prometheus is disposable. Dashboard and alert bundle changes require `promtool` lint and provisioning smoke before merge. Non-loopback exporter bind requires explicit operator acknowledgement and an approved auth control; rollback must not silently widen exposure.

## Artifacts/evidence

- API route inventory, metrics schema report, cardinality proof, exporter lifecycle trace, `promtool` lint output, Grafana provisioning smoke log, mixed-client interop matrix, Hive results, synthetic alert fire/resolution log, and telemetry budget report in `artifacts/phase-12/`.

## Dependencies

Depends on Phases 03–11 and [../06-observability/](../06-observability/). Phase 13 consumes observability gates for release qualification and chaos/soak evidence.
