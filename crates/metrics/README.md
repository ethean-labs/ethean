# ethean-metrics

In-process `ethean_` Prometheus registry with closed labels and readiness gates.

- Schema `ethean-metrics-v1` (OSD-009 leanMetrics upstream pin still open)
- Forbidden labels: root, peer_id, validator_index, signature
- Exporter HTTP: `/metrics`, `/healthz`, `/readyz` via `spawn_metrics_server`
- Slot gauges: `head_slot`, `justified_slot`, `finalized_slot`, `slot_current`
