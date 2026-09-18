# Prometheus Topology

How Prometheus discovers, scrapes, and retains metrics from Ethean nodes, peer clients, and host exporters. Complements [METRICS_CONTRACT.md](METRICS_CONTRACT.md) and [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md).

## Supported deployment shapes

### Local developer (single node)

```text
┌──────────────┐     scrape      ┌─────────────┐
│ ethean node  │◄────────────────│ Prometheus  │
│ :9090/metrics│                 │  (pinned)   │
└──────────────┘                 └──────┬──────┘
                                      │ PromQL
                               ┌──────▼──────┐
                               │ Grafana     │
                               │ (pinned)    │
                               └─────────────┘
```

- Static target: `127.0.0.1:9090` in `deploy/observability/prometheus/targets/local.yml`.
- Loopback bind on Ethean; no auth required.
- Ephemeral TSDB (tmpfs or `./data/prometheus` gitignored).

### Multi-node local lab (2+ Ethean + peers)

Used for mixed-client interop and Phase 12 qualification.

```text
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│ethean-0 │ │ethean-1 │ │ ream-0  │ │ zeam-0  │
│ :9090   │ │ :9090   │ │ :metrics│ │ :metrics│
└────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘
     │           │           │           │
     └───────────┴───────────┴───────────┘
                         │
                  ┌──────▼──────┐
                  │ Prometheus  │
                  │ static or   │
                  │ file_sd     │
                  └─────────────┘
```

**Target generation options (pick one per profile):**

| Method | When | Source |
|---|---|---|
| Generated static targets | Fixed local devnet, CI, Hive | `scripts/gen-prometheus-targets.sh` from inventory JSON |
| File-based service discovery (`file_sd`) | Docker Compose / k8s sidecars | Atomically written `targets/*.json` |
| DNS-based SD | Named services in Compose network | `dns_sd_configs` with SRV or A records |

Inventory JSON schema (generated, not hand-edited):

```json
{
  "snapshot": "pq-devnet-0-pin-abc123",
  "targets": [
    {
      "address": "ethean-0:9090",
      "labels": {
        "client": "ethean",
        "node": "ethean-0",
        "role": "validator",
        "network": "devnet-local"
      }
    }
  ]
}
```

### Hive / devnet orchestration

Hive (or equivalent test harness) provisions N nodes with deterministic names. Prometheus runs in a **sidecar container** on the same Docker network:

1. Hive startup writes `/prometheus/file_sd/hive.json` atomically (write temp + rename).
2. Prometheus `file_sd` reload picks up new targets within `refresh_interval: 30s`.
3. Each target carries labels: `client`, `node`, `role`, `snapshot`, `network`, `hive_run_id`.
4. `hive_run_id` is bounded (run UUID); used for test correlation, not production labels on node metrics.

Scrape **all** exporters:

| Job name | Targets | Notes |
|---|---|---|
| `ethean` | Ethean nodes | Primary contract ([METRICS_CONTRACT.md](METRICS_CONTRACT.md)) |
| `peer-clients` | Ream, Zeam, etc. | Relabel to common dashboard variables; retain native metric names |
| `node-exporter` | Host metrics | Optional per-VM; label `instance` matches inventory |
| `cadvisor` | Container metrics | Optional in Compose profiles |

Peer client scrape paths and ports come from Phase 00 evidence ledger; do not guess.

### CI smoke topology

Ephemeral profile (no persistent volumes):

- Pinned Prometheus + Grafana containers from digest-pinned images.
- Minimum fixture: 2 Ethean nodes + 1 peer client stub or real peer at pinned commit.
- Pre-seed synthetic metrics via `prometheus/testdata/` for rule tests when nodes are not fully built.
- Tear down after: verify `up` goes to 0, TSDB discarded.

### Staging / production fleet

Each failure domain runs a local Prometheus. Global query tier optional for long-term aggregates; **node-level alerting must not depend solely on the global tier**.

Remote write is asynchronous and bounded; its outage cannot block local scraping or Ethean.

## Version-pinned containers

Images are pinned by **immutable digest** in `deploy/observability/versions.lock`:

| Component | Pin fields | Upgrade gate |
|---|---|---|
| Prometheus | `image`, `digest`, `version` | `promtool check config`, rule tests, soak |
| Grafana | `image`, `digest`, `version` | provisioning smoke, dashboard JSON lint |
| Alertmanager | `image`, `digest`, `version` | route/template validation |
| node-exporter | optional | host dashboard compatibility |

Example manifest fragment (values illustrative; real digests set in Phase 12):

```yaml
# deploy/observability/versions.lock.yaml
prometheus:
  image: prom/prometheus
  version: v2.54.1
  digest: sha256:…
grafana:
  image: grafana/grafana
  version: 11.2.0
  digest: sha256:…
```

Security gate G0 requires these pins in CI artifacts ([../04-risks/SECURITY_GATES.md](../04-risks/SECURITY_GATES.md)).

## Healthchecks

Container healthchecks (Compose / k8s):

| Service | Probe | Pass |
|---|---|---|
| Prometheus | `GET /-/healthy` | 200 |
| Prometheus ready | `GET /-/ready` | 200 ( WAL replay done ) |
| Grafana | `GET /api/health` | 200 |
| Ethean exporter | `GET /readyz` on metrics addr | 200 when node ready |
| Ethean liveness | `GET /healthz` | 200 while event loop alive |

Prometheus **`up`** metric is the primary scrape-health signal. Target labels must survive `up == 0` long enough for absence alerts ([ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md)).

## Ephemeral vs persistent lab profiles

| Profile | TSDB retention | Grafana DB | Use |
|---|---|---|---|
| `lab-ephemeral` | 2 h | ephemeral | CI, Hive runs, local debug |
| `lab-persistent` | 7 d | persistent volume | Soak, dashboard development |
| `staging` | 15 d | persistent | Pre-release qualification |
| `production` | per SLO doc | HA Grafana | Operator fleet |

Switch profile via `OBSERVABILITY_PROFILE=lab-ephemeral` (env) or Compose override file.

## Exporter endpoint (Ethean)

Dedicated address; does not share public Beacon/API listener.

| Path | Purpose |
|---|---|
| `/metrics` | Prometheus text exposition |
| `/healthz` | Process health |
| `/readyz` | Role readiness |

Defaults ([METRICS_CONTRACT.md](METRICS_CONTRACT.md)):

- Loopback bind (`127.0.0.1:9090`).
- Non-loopback disabled until `metrics.unsafe_expose=true` plus auth control.
- No debug, profile, config, or secret endpoints on metrics listener.
- Bind error is startup failure when metrics required.
- Duplicate metric registration always fatal.

### Production exporter safe bind defaults

| Setting | Production default | Rationale |
|---|---|---|
| `listen_addr` | `127.0.0.1:9090` | No accidental exposure |
| `unsafe_expose` | `false` | Explicit opt-in |
| TLS | via sidecar or `metrics.tls` | Required for non-loopback |
| Scrape from | same-host Prometheus agent or authenticated proxy | No public `/metrics` |

Operators who scrape from another host use:

1. **Node-local Prometheus agent** (recommended) → remote_write to central, or
2. **Authenticated reverse proxy** terminating TLS in front of loopback exporter, or
3. **mTLS on exporter** with client CA pinned to scraper identity.

## Target identity and relabeling

Deployment-owned labels (never from target-supplied metric labels):

| Label | Example | Owner |
|---|---|---|
| `environment` | `ci`, `devnet`, `staging` | inventory |
| `cluster` | `hive-run-42` | inventory |
| `client` | `ethean`, `ream`, `zeam` | inventory |
| `node` | `ethean-0` | inventory |
| `role` | `validator`, `bootnode`, `aggregator` | inventory |
| `snapshot` | `pq-devnet-0-abc123` | pin manifest |
| `network` | `devnet-local` | profile |
| `instance` | `ethean-0.devnet-local` | stable ops ID, **not** peer ID |

Relabeling rules:

- Drop all discovery metadata not in allowlist.
- Reject targets whose `network` or `snapshot` does not match deployment pin.
- Mixed-client jobs retain `client` and `metrics_schema` for dashboard filters.
- Ethean-specific recording rules select `client="ethean"` and compatible `metrics_schema`.

Example relabel (static job):

```yaml
relabel_configs:
  - source_labels: [__address__]
    target_label: instance
    replacement: ${node}.${network}
  - target_label: client
    replacement: ethean
```

## Scrape behavior

Parameters derived in Phase 00 from slot resolution and exporter cost ([../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md)):

| Parameter | Guidance |
|---|---|
| `scrape_interval` | Must resolve single-slot events for critical counters; typical 1–4 s in lab, 5–15 s in production |
| `scrape_timeout` | `< scrape_interval`; `<` exporter render p99; must not consume material portion of 4 s slot |
| `sample_limit` | Second defense against cardinality blowout |
| `label_limit` / `label_length_limit` | Enforce contract |
| `body_size_limit` | From Phase 00 exposition size p99 × margin |

Example global defaults (lab profile):

```yaml
global:
  scrape_interval: 4s
  scrape_timeout: 3s
  evaluation_interval: 4s
```

Critical security counters (`xmss_reuse`, `xmss_rollback`, deadline misses) require temporal resolution to catch a single slot event. This informs interval choice; it does not justify skipping performance validation.

## Recording and retention

Recording rules precompute ([ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md)):

- Fleet slot/head/finality rates and lag
- Histogram quantiles from `_bucket` (never average node-side quantiles)
- Duty miss rates by role
- Prover queue saturation
- Sync distance and peer connectivity
- Storage pressure

Retention:

| Tier | Raw resolution | Duration |
|---|---|---|
| Lab ephemeral | full | 2 h |
| Lab persistent | full | 7 d |
| Production local | full | 15–30 d |
| Production aggregated | 5 m downsampled | 90 d |

Signing audit remains durable application data; it must not rely on Prometheus retention.

## Availability monitoring

Monitor independently of workload alerts:

- `up`, `scrape_duration_seconds`, `scrape_samples_scraped`
- Target churn (unexpected count change)
- Rule evaluation failures and duration
- TSDB compaction errors, WAL corruption, disk usage
- Remote-write queue depth and drops
- Alertmanager notification failures
- Grafana datasource health

Prometheus self-failure must be observed by an out-of-band check ( sibling Prometheus, blackbox, or cloud monitor).

## Security

- Metrics endpoints and Prometheus/Grafana UIs are not internet-exposed by default.
- Network policy restricts scrapers and operator access.
- Grafana uses least-privilege datasource credentials; anonymous admin disabled.
- Config backups exclude secret material.
- Query URLs and dashboard links must not carry credentials.
- Generated target files may list addresses but never scrape passwords.

## CI smoke topology checklist

Phase 12 CI job `observability-smoke`:

1. Start pinned Prometheus + Grafana from digest lock.
2. Start 2 Ethean nodes + 1 peer (or metric stub) with generated targets.
3. Wait until all `up == 1` and Ethean `/readyz == 200`.
4. Assert sample count within budget; run `promtool check rules`.
5. Confirm Grafana datasources and dashboards auto-provisioned.
6. Fire synthetic alert (inject metric or use `promtool test rules`).
7. Stop nodes; confirm `up == 0` within 2 intervals; exporter port released.

See [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md) for exit-gate scenarios.
