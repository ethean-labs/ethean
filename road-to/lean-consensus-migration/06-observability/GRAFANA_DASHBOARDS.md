# Grafana Dashboards

All dashboards are **provisioned from source control**. Operators do not build production dashboards in the Grafana UI. Manual edits in running Grafana are considered ephemeral; JSON in the repo is authoritative.

## Provisioning layout (planned)

```text
deploy/observability/grafana/
├── provisioning/
│   ├── datasources/prometheus.yml
│   └── dashboards/default.yml
├── dashboards/
│   ├── fleet-overview.json
│   ├── consensus-finality.json
│   ├── validator-duties.json
│   ├── xmss-leanvm-prover.json
│   ├── p2p-quic-gossip.json
│   ├── storage-sync-recovery.json
│   ├── host-resources.json
│   └── mixed-client-interop.json
└── README.md
```

CI validates JSON syntax, required template variables, and that every panel query parses against recording rules in [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md).

## Global template variables

Present on all multi-node dashboards:

| Variable | Source | Multi-select |
|---|---|---|
| `datasource` | Prometheus provisioning | no |
| `environment` | label `environment` | yes |
| `network` | label `network` | yes |
| `snapshot` | label `snapshot` | yes |
| `client` | label `client` | yes |
| `node` | label `node` | yes |
| `role` | label `role` | yes |

Single-node mode: default `node` to localhost inventory entry. Multi-node mode: `All` enabled with `$__all` cap (max 32 series per panel; overflow shows warning panel).

### Compare mode

`compare_clients` custom variable: when enabled, panels duplicate queries grouped by `client` instead of `node`. Used on [mixed-client-interop.json](#mixed-client-interop) only by default; optional on consensus and P2P dashboards.

## Dashboard inventory

### 1. Fleet overview (`fleet-overview.json`)

**Audience:** operator on-call, CI smoke review.

**Panels:**

- Target health: `up` by `client`, `node`
- Head slot vs wall clock (4 s slot grid)
- Finality lag (`ethean_finality_lag_slots` / recording rule)
- Active alerts (Grafana alert list or Alertmanager annotation)
- Scrape duration and sample count
- Build info matrix: `metrics_schema`, protocol pin, leanMetrics pin

**Filters:** all global variables.

### 2. Consensus and finality (`consensus-finality.json`)

**Panels:**

- Head / justified / finalized / safe-target slot timelines (stacked step chart on 4 s slot axis)
- `finalized_change_total` rate by reason
- Reorg count and depth histogram heatmap
- Finality stall gauge (`finality_stall_seconds`)
- Block and attestation processing by `result`
- Clock offset vs slot phase overlay

**Correlation row:** mark intervals where `finality_stall_seconds > threshold` AND `reorg_total` increases.

### 3. Validator duties (`validator-duties.json`)

**Panels:**

- Duty assigned / completed / missed rates by `role`, `duty`
- Milestone histograms: prepare → sign → prove → publish
- Deadline miss counter by `stage`, `role`
- Missed proposal and attestation tables (aggregated counts only, no validator ID)
- Slot timeline with duty markers (vertical lines at slot boundaries every 4 s)

**Single-node focus:** default filter one validator node. **Multi-node:** compare miss rates across nodes.

### 4. XMSS and leanVM prover (`xmss-leanvm-prover.json`)

**Panels:**

- Leaves remaining by `role` (low-leaf warning band)
- Leaf burned / exhaustion / security incident counters
- Crypto operation duration heatmap (`operation`, `result`)
- Prover queue items/bytes vs active workers
- Job duration p50/p95/p99 from histogram buckets
- Proof coverage ratio by `proof_class`
- Proof input/output byte distributions
- Deadline miss and timeout counters
- Worker heartbeat age

**Correlation row:** prover wedge — queue bytes ↑, heartbeat age ↑, duty miss ↑.

### 5. P2P, QUIC, and gossip (`p2p-quic-gossip.json`)

**Panels:**

- Connected peers by `direction`, `client_family`
- Connection/disconnect rates by `reason`
- Gossip received/validated/published by `topic`
- Validation and propagation latency by `topic`
- Message size histograms
- Queue drops by `queue`, `reason`
- QUIC RTT and loss ratio (aggregated)
- Req-resp duration and error rate by `protocol`

**Correlation row:** partition hint — peer count ↓ on subset of nodes, gossip received ↓, req-resp errors ↑.

### 6. Storage, sync, and recovery (`storage-sync-recovery.json`)

**Panels:**

- Sync state and distance in slots
- Checkpoint import outcomes and age
- Storage operation latency by `operation`
- Commit/recovery/corruption counters
- Logical and disk bytes; open FDs
- Migration phase gauge
- Prune operation rate

**Correlation row:** recovery — `storage_recovery_total` spike, `ready{component="storage"}` flapping, head lag.

### 7. Host resources (`host-resources.json`)

**Panels:**

- CPU usage (process + node-exporter if present)
- RSS and cgroup memory limit headroom
- Thread count by `pool`
- Internal task queue depth
- Open FDs vs ulimit
- Disk I/O and free space (node-exporter)
- Network bytes (host level)

Pairs with [../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md) exporter regression panel.

### 8. Mixed-client interop (`mixed-client-interop.json`)

**Audience:** Phase 12 / Phase 13 qualification.

**Panels:**

- Head slot delta: `max(head_slot) - min(head_slot)` by `snapshot` across `client`
- Finalized slot delta (same)
- Finality lag comparison by `client`
- Gossip validation reject rate comparison (normalized per client metric mapping)
- Req-resp error rate comparison
- Duty miss rate comparison (Ethean only unless peer exposes equivalent)
- Table: build pins per `client`, `node`

Requires Phase 00 peer metric mapping table in provisioning README. Unknown peer metrics show "not exposed" panel instead of broken query.

## Query conventions

1. Prefer recording rules (`:rate5m`, `:quantile99`) over raw histograms in dashboards.
2. Use `histogram_quantile` only on aggregated `_bucket` series from Prometheus recording rules.
3. Slot-aligned charts use 4 s minimum interval; x-axis sync across row for correlation.
4. No query may introduce high-cardinality label matchers (`peer_id`, roots, etc.).
5. Panel max data points capped; use recording rules for long ranges.
6. Unit metadata matches [METRICS_CONTRACT.md](METRICS_CONTRACT.md) (`s`, `bytes`, `short`).

Example slot timeline query:

```promql
max by (node) (ethean_head_slot{client="ethean", network="$network", snapshot="$snapshot"})
```

Example finality lag (recording rule):

```promql
ethean:finality_lag_slots{client="ethean", node=~"$node"}
```

## 4 s slot timeline

All time-series panels that represent consensus progress include:

- **Interval:** 4 s (or `$__interval` ≥ 4 s)
- **Axis options:** shared crosshair across dashboard rows
- **Annotations:** optional slot boundaries from recording rule `ethean:slot_tick` (1 every 4 s from max head movement)

Duty dashboards overlay:

- Red band: deadline miss events (`increase(ethean_slot_deadline_miss_total[4s]) > 0`)
- Orange band: prover queue above saturation threshold

## Correlation dashboards (built-in rows)

| Scenario | Signals | Dashboard |
|---|---|---|
| Finality stall | `finality_stall_seconds` ↑, finalized rate = 0, head may still move | consensus-finality |
| Prover wedge | queue bytes ↑, heartbeat age ↑, proof timeout ↑, duty miss ↑ | xmss-leanvm-prover + validator-duties |
| Partition | peer count split across nodes, gossip ↓, req-resp errors ↑ | p2p-quic-gossip + fleet-overview |
| Sync stall | sync_distance flat, head behind fleet max | storage-sync-recovery |
| XMSS exhaustion | leaves remaining → 0, exhaustion counter ↑ | xmss-leanvm-prover |

Each correlation row links to relevant alerts in [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md).

## Single vs multi-node filters

| Mode | Default variables | Behavior |
|---|---|---|
| Single | `node=ethean-0`, one `client` | Full duty and prover detail |
| Multi homogeneous | `client=ethean`, `node=All` | Fleet aggregates + per-node small multiples |
| Multi mixed | `client=All`, `compare_clients=true` | Interop dashboard primary; consensus lag deltas |

## Provisioning config (example)

```yaml
# deploy/observability/grafana/provisioning/dashboards/default.yml
apiVersion: 1
providers:
  - name: ethean
    orgId: 1
    folder: Ethean
    type: file
    disableDeletion: true
    updateIntervalSeconds: 30
    options:
      path: /etc/grafana/dashboards
```

```yaml
# deploy/observability/grafana/provisioning/datasources/prometheus.yml
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

## CI and review gates

- `grafana-dashboard-lint` job: JSON schema, duplicate UIDs, variable coverage.
- Snapshot test: render panel queries with `promtool query instant` against fixture TSDB.
- Dashboard changes require observability reviewer when recording rules or metric names change.
- Dashboard UID and version bump on breaking panel removal.

## Related documents

- [METRICS_CONTRACT.md](METRICS_CONTRACT.md) — metric names and labels
- [PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md) — datasource URL and scrape labels
- [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md) — recording rules consumed by panels
- [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md) — scenarios exercised on these dashboards
