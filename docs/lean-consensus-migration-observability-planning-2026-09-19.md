# Lean Consensus Migration — Observability Planning Pack

**Date:** 2026-09-19  
**Scope:** Planning documents only (no product code).  
**Location:** [road-to/lean-consensus-migration/06-observability/](../road-to/lean-consensus-migration/06-observability/)

## Summary

Created the full observability planning folder for Phase 12 (Observability and fleet qualification). Six English Markdown documents define the `ethean-metrics` contract, Prometheus scrape topology, provisioned Grafana dashboards, alerting/SLOs, and multi-node exit gates.

## Files

| File | Purpose |
|---|---|
| `README.md` | Index, reading order, Phase 12 / `ethean-metrics` relation, architecture, delivery gates |
| `METRICS_CONTRACT.md` | leanMetrics vs `ethean_` namespace, exporter lifecycle, metric families, cardinality budgets |
| `PROMETHEUS_TOPOLOGY.md` | Local/Hive multi-node targets, pinned containers, healthchecks, lab profiles |
| `GRAFANA_DASHBOARDS.md` | Eight provisioned dashboards, variables, 4 s slot timeline, correlation rows |
| `ALERTING_AND_SLOS.md` | SLOs, recording/alert rules, promtool CI, synthetic fault tests |
| `MULTINODE_ANALYSIS.md` | 2+ Ethean + 1 peer scenarios, Hive workflow, Phase 12 exit checklist |

## Key decisions

- **leanMetrics first:** use pinned upstream names when available; otherwise versioned `ethean_` prefix with `metrics_schema` in build info.
- **High-cardinality forbidden:** no validator ID, peer ID, roots, message IDs, or proof bytes as labels; CI enforces series budget.
- **Exporter at node start:** dedicated listener, readiness gating by component, graceful shutdown with scrape grace, loopback default bind.
- **Source-controlled ops:** Grafana dashboards and Prometheus rules are provisioned from repo; UI edits are not authoritative.
- **Phase 12 exit gate:** healthy scrape of Ethean + peers, auto-provisioned Grafana, synthetic finality-stall / prover-wedge / peer-loss alerts fire in CI or Hive.

## Dependencies

- Phases 03–11 (consensus, crypto, P2P, storage) supply instrumentation points.
- Phase 00 baselines required before numeric scrape intervals, SLO targets, and histogram buckets are finalized.
- `ethean-metrics` crate layout defined in [TARGET_WORKSPACE.md](../road-to/lean-consensus-migration/03-architecture/TARGET_WORKSPACE.md).

## Next implementation steps (Phase 12)

1. Create `crates/ethean-metrics` with registry, recorders, and exporter.
2. Add `deploy/observability/` with pinned Compose, Prometheus config, rules, and Grafana JSON.
3. Wire CI: golden exposition fixture, promtool checks, rule unit tests, multinode smoke job.
