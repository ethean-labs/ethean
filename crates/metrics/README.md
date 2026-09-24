# ethean-metrics

In-process `ethean_` Prometheus registry with closed labels and readiness gates.

- Schema `ethean-metrics-v4`: `ethean_*` families plus the leanMetrics `lean_*` table (pinned commit 69f9722) and the leanSpec node-registry coverage gauges
- Forbidden labels: root, peer_id, validator_index, signature
- Exporter HTTP: `/metrics`, `/healthz`, `/readyz` via `spawn_metrics_server`
- Slot gauges: `head_slot`, `justified_slot`, `finalized_slot`, `safe_target_slot`, `slot_current`
- FC: `fc_reorg_total` (counter; wiring incremental)
- Name map vs leanMetrics: see `docs/leanmetrics-safe-target-name-map-2026-09-20.md`
