# Lean Consensus Observability

Observability is a release control for consensus safety and liveness, not an optional deployment add-on. It must prove the risk and performance gates in [../04-risks/](../04-risks/README.md) without leaking secrets or destabilizing the node.

## Relation to Phase 12 and `ethean-metrics`

**Phase 12 — Observability and fleet qualification** (planned; see [../phases/10-quic-gossip-and-reqresp.md](../phases/10-quic-gossip-and-reqresp.md) and [../phases/11-storage-sync-and-checkpoints.md](../phases/11-storage-sync-and-checkpoints.md)) delivers the production telemetry surface. Phase 12 depends on Phases 03–11 and is the last gate before Phase 13 release qualification.

Phase 12 implementation scope:

| Deliverable | Crate / artifact | Planning doc |
|---|---|---|
| Metric names, recorders, registry | `ethean-metrics` ([../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md)) | [METRICS_CONTRACT.md](METRICS_CONTRACT.md) |
| Prometheus exporter lifecycle at node start | `ethean-metrics` + `ethean` runtime wiring | [METRICS_CONTRACT.md](METRICS_CONTRACT.md) § Exporter lifecycle |
| Scrape topology, pins, discovery | `deploy/observability/` (planned) | [PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md) |
| Provisioned Grafana dashboards | `deploy/observability/grafana/` (planned) | [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md) |
| Recording/alert rules + CI lint | `deploy/observability/prometheus/rules/` (planned) | [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md) |
| Multi-node / mixed-client analysis | CI + Hive fixtures | [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md) |

`ethean-metrics` is observational only: consensus decisions must not branch on recorder success or metric values ([../03-architecture/DEPENDENCY_RULES.md](../03-architecture/DEPENDENCY_RULES.md)). Instrumentation lives in owning crates; `ethean-metrics` owns registration, cardinality enforcement, and export.

## Baseline and evidence

Ethean currently has in-process statistics and synthetic performance structures, but no Prometheus dependency, exporter lifecycle, provisioned Grafana stack, alert rules, or multi-node analysis contract in `Cargo.toml` and the inspected source. Existing synthetic values are not production telemetry.

Peer clients—Ream, Zeam, qlean-mini, ethlambda, Lantern, gean, and Peam—must be sampled at immutable commits during Phase 00. The evidence ledger records exposed names, semantics, labels, readiness behavior, topology, and protocol-specific signals. Common concepts may be mapped into fleet dashboards, but Ethean does not copy peer naming blindly. **leanMetrics** is preferred when a pinned version defines the metric; otherwise Ethean uses the versioned `ethean_` namespace ([../02-protocol/UPSTREAM_REFRESH_POLICY.md](../02-protocol/UPSTREAM_REFRESH_POLICY.md)).

No local peer-research Markdown or live external repository access was available while this plan was authored. Phase 00 evidence capture is therefore a mandatory first gate, and this document does not claim unsupported peer behavior.

## Reading order

1. **[METRICS_CONTRACT.md](METRICS_CONTRACT.md)** — what Ethean exposes, label policy, cardinality budgets, exporter lifecycle. Read first; all other docs assume this contract.
2. **[PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md)** — how Prometheus discovers and scrapes nodes locally, in Hive/devnet, and in production.
3. **[GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md)** — provisioned dashboard inventory, variables, and correlation panels.
4. **[ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md)** — SLO definitions, recording rules, alert rules, and CI validation.
5. **[MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md)** — fleet scenarios, mixed-client comparison, Phase 12 exit gate.

Skim [../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md) and [../04-risks/SECURITY_GATES.md](../04-risks/SECURITY_GATES.md) before approving scrape intervals or alert thresholds.

## Document map

| Document | Purpose |
|---|---|
| [METRICS_CONTRACT.md](METRICS_CONTRACT.md) | Names, types, labels, cardinality, exporter lifecycle |
| [PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md) | Exporter security, scrape topology, discovery, retention, version pins |
| [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md) | Provisioned dashboard inventory and query conventions |
| [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md) | Actionable alerts, SLO measurement, fault validation |
| [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md) | Fleet, mixed-client, divergence, and propagation analysis |

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│ Ethean node (ethean binary)                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │ chain/       │  │ network/     │  │ validator/prover/      │ │
│  │ storage/     │  │ recorders    │  │ storage recorders      │ │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬────────────┘ │
│         └─────────────────┴──────────────────────┘              │
│                           │                                     │
│                    ethean-metrics registry                      │
│                           │                                     │
│              HTTP : metrics / healthz / readyz                  │
└───────────────────────────┼─────────────────────────────────────┘
                            │ scrape (TLS/mTLS or loopback)
                            ▼
                   ┌─────────────────┐
                   │ Prometheus      │◄── recording rules
                   │ (pinned image)  │◄── alert rules → Alertmanager
                   └────────┬────────┘
                            │ PromQL
                            ▼
                   ┌─────────────────┐
                   │ Grafana         │  provisioned dashboards only
                   │ (pinned image)  │
                   └─────────────────┘
```

Instrumentation records bounded events close to the owning subsystem. A central registry renders Prometheus text format through a dedicated HTTP listener. Prometheus scrapes node exporters and host/container exporters. Recording rules normalize expensive queries. Grafana reads Prometheus only; it never queries node internals directly. Alertmanager routing is deployment-owned; alert rule semantics and tests live with Ethean.

## Exporter lifecycle (summary)

Full detail in [METRICS_CONTRACT.md](METRICS_CONTRACT.md).

**Startup order:**

1. Validate metric descriptors; reject duplicate or incompatible registrations.
2. Parse bind address, TLS/auth integration, and exposure policy.
3. Bind the listener; bind failure is fatal when metrics are required.
4. Start exporter serving `/metrics`, `/healthz`, and `/readyz`.
5. Keep readiness false until storage recovery, protocol/crypto pin validation, network initialization, and required signing-state checks finish.
6. Set readiness true only when the node can perform its configured role.

**Shutdown order:**

1. Set readiness false.
2. Reject new work and stop new exporter connections.
3. Allow an in-flight scrape to finish within the benchmark-derived grace interval.
4. Flush final counters where supported.
5. Close the listener and join the exporter task.

`/healthz` reports process/event-loop health. `/readyz` reports role readiness. Neither includes secrets or detailed internal state.

## Exposure and authentication

Default bind is loopback on a separately configured metrics address. Non-loopback bind requires an explicit unsafe-exposure acknowledgement plus one approved control: exporter TLS/client authentication, an authenticated reverse proxy, or a private network policy. Public unauthenticated metrics fail deployment policy. Scrape credentials are read from protected files or secret stores, never CLI arguments, labels, logs, dashboards, or generated config committed to source.

## Phase 12 delivery gates

Phase 12 exit requires all items below plus the scenarios in [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md):

- Metric schema and cardinality tests in `ethean-metrics` CI.
- Exporter startup, bind conflict, readiness transition, slow scrape, disconnect, auth, and shutdown tests.
- `promtool check config` and `promtool check rules` using the pinned Prometheus image.
- Rule unit tests with synthetic series ([ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md)).
- Grafana provisioning smoke test using its pinned image and a seeded Prometheus.
- Homogeneous and mixed-client multi-node scrape test.
- Synthetic crash, partition, prover hang, disk pressure, leaf exhaustion, stale checkpoint, decode flood, and clock-skew validation.
- Performance test proving telemetry stays inside Phase 00-derived CPU, memory, and latency budgets ([../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md)).

Prometheus and Grafana versions are exact image digests in deployment manifests. Upgrade pull requests include release-note review, config/rule/dashboard lint, migration notes, and smoke results.

## Related documents

- [../04-risks/README.md](../04-risks/README.md) — risk register and gates telemetry must satisfy.
- [../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md) — `ethean-metrics` crate layout.
- [../03-architecture/DEPENDENCY_RULES.md](../03-architecture/DEPENDENCY_RULES.md) — who may depend on metrics.
- [../02-protocol/UPSTREAM_REFRESH_POLICY.md](../02-protocol/UPSTREAM_REFRESH_POLICY.md) — leanMetrics refresh rules.
