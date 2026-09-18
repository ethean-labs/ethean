# Multi-Node Analysis

Scenarios and acceptance tests for fleet observability: homogeneous Ethean clusters, mixed-client devnets, Hive runs, and Phase 12 exit qualification.

Depends on [METRICS_CONTRACT.md](METRICS_CONTRACT.md), [PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md), [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md), and [ALERTING_AND_SLOS.md](ALERTING_AND_SLOS.md).

## Minimum topology

Phase 12 lab fixture:

| Node | Client | Role | Purpose |
|---|---|---|---|
| `ethean-0` | ethean | validator + proposer | primary subject |
| `ethean-1` | ethean | validator + attester | homogeneous comparison |
| `peer-0` | ream *or* zeam (pinned) | validator | mixed-client interop |

*Peer choice follows Phase 00 evidence at pinned commit; do not float.*

Optional extensions:

- `ethean-2` as aggregator-only role
- `bootnode-0` as non-validator for P2P-only metrics
- Host `node-exporter` on each VM

## Analysis scenarios

### S1 — Healthy fleet baseline

**Setup:** 2 Ethean + 1 peer, pinned snapshot, all nodes synced.

**Collect:** 30 min lab-persistent profile scrape at 4 s interval.

**Assert:**

- All targets `up == 1`; Ethean `/readyz == 200`.
- `max(head_slot) - min(head_slot) <= 1` across Ethean nodes.
- Finalized slot equal across Ethean nodes.
- Peer head/finalized within interop tolerance (Phase 00 table).
- Grafana dashboards auto-load; variables populate.
- Sample count per Ethean target within cardinality budget.

**Dashboards:** fleet-overview, mixed-client-interop.

### S2 — Head divergence detection

**Setup:** Partition `ethean-1` from peers for 2 min; heal.

**Correlate across clients:**

| Signal | ethean-0 | ethean-1 | peer-0 |
|---|---|---|---|
| `head_slot` | advances | stalls then jumps | advances |
| `peers_connected` | stable | drops | stable |
| `gossip_received_total` rate | stable | drops | stable |
| `reqresp_error_total` | low | spike on heal | low |

**Assert:** `EtheanStaleHead` or `EtheanPeerLoss` fires on partition; clears after heal. Mixed-client dashboard shows head delta spike.

### S3 — Finality stall correlation

**Setup:** Inject condition where blocks arrive but finality does not advance (test hook or misconfigured superminority).

**Correlate:**

- `finality_stall_seconds` ↑ on all Ethean nodes
- `finalized_change_total` rate → 0
- `head_slot` may still increase
- Proposer duty panels normal; attester misses may increase

**Assert:** `EtheanFinalityStall` fires on all scraped Ethean targets within alert `for` window.

**Dashboards:** consensus-finality correlation row.

### S4 — Prover wedge

**Setup:** aggregator node (`ethean-2` or role on `ethean-0`) with prover hang fault.

**Correlate:**

- `prover_queue_bytes` ↑
- `prover_worker_heartbeat_age_seconds` ↑
- `prover_jobs_total{result="timeout"}` ↑
- `duty_missed_total{role="aggregator"}` ↑
- Attester duties on other nodes may complete; aggregator publish ↓

**Assert:** `EtheanProverWorkerStale`, `EtheanProofTimeout`, `EtheanProverQueueSaturated` fire in order.

**Dashboards:** xmss-leanvm-prover + validator-duties.

### S5 — Peer loss and partition

**Setup:** Drop 50%+ peers on `ethean-1` via network filter.

**Correlate:**

- `peers_connected` drop on affected node only
- Fleet min peer count asymmetry → partition suspect
- Gossip propagation latency ↑ on affected node
- Req-resp error rate ↑

**Assert:** `EtheanPeerLoss` warning; `EtheanPartitionSuspect` if asymmetric pattern matches rule.

### S6 — Sync and recovery

**Setup:** Start `ethean-1` from checkpoint behind fleet; restart `ethean-0` cold.

**Correlate:**

- `sync_distance_slots` ↓ over time on catching node
- `storage_recovery_total` on restart
- `ready{component="storage"}` sequence 0 → 1
- `EtheanRestartRecoverySlow` only if exceeds Phase 00 p95

**Dashboards:** storage-sync-recovery.

### S7 — XMSS leaf pressure

**Setup:** Accelerated signing workload in test profile (not production keys).

**Correlate:**

- `xmss_leaves_remaining` ↓
- `xmss_leaf_burned_total` ↑
- `EtheanXmssLeafLifetime` warning before exhaustion

**Assert:** no false positive on `EtheanXmssReuseDetected`.

### S8 — Mixed-client metric comparison

**Setup:** S1 baseline extended 1 h.

**Compare across `client` label:**

| Concept | Ethean metric | Peer mapping (Phase 00) |
|---|---|---|
| Head slot | `ethean_head_slot` | peer-specific name |
| Finalized slot | `ethean_finalized_slot` | peer-specific name |
| Peer count | `ethean_peers_connected` | peer-specific name |
| Missed duties | duty miss counters | if exposed |

**Assert:** dashboard `mixed-client-interop` shows delta panels; deltas within pinned interop tolerance. Document peer gaps in evidence ledger; do not fail on unexposed optional metrics.

## Hive / devnet workflow

1. Hive test definition starts N clients + Prometheus sidecar ([PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md)).
2. `hive_run_id` label applied to all targets for the run.
3. Post-run artifact bundle:
   - `prometheus-tsdb-snapshot/` (optional, size-capped)
   - `grafana-dashboard-screenshot/` or JSON panel export
   - `alertmanager-log.json` for fired alerts
   - `targets.json` used for file_sd
4. CI uploads bundle as Phase 12 evidence.

## Cross-client correlation method

When comparing Ethean and peers:

1. **Align time** on slot boundaries (4 s), not wall clock skew.
2. **Join on** `network`, `snapshot`, and role-equivalent labels only.
3. **Normalize rates** per client using recording rules where names differ.
4. **Never** join on peer ID, validator index, or block root.
5. Log divergences > tolerance to evidence manifest with pinned commits.

Example PromQL (Ethean head vs fleet max):

```promql
ethean_head_slot{client="ethean", node="ethean-0"}
  - max(ethean_head_slot{client="ethean"})
```

Peer comparison uses mapped recording rules, e.g. `interop:head_slot:max{client="ream"}`.

## Phase 12 exit gate

Phase 12 is complete when **all** gates pass on the minimum topology (2 Ethean + 1 peer) in CI or signed Hive run:

### Scrape and stack health

- [ ] Prometheus scrapes all Ethean and peer exporters with labels `client`, `node`, `role`, `snapshot`
- [ ] Generated static targets or file_sd reload verified
- [ ] Pinned Prometheus + Grafana containers pass healthchecks
- [ ] `promtool check config` and `promtool check rules` green

### Grafana provisioning

- [ ] All dashboards in [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md) auto-provisioned (no manual UI setup)
- [ ] Template variables work for single- and multi-node selection
- [ ] Mixed-client interop dashboard renders with peer metrics or explicit "not exposed"

### Synthetic alerts fire

Inject or simulate faults; alerts must fire and recover:

- [ ] Finality stall (`S3`)
- [ ] Proof timeout / prover wedge (`S4`)
- [ ] Peer loss (`S5`)

Optional but recommended in lab-persistent soak:

- [ ] Restart recovery slow boundary (`S6`)
- [ ] Disk pressure simulation

### Performance and safety

- [ ] Exporter scrape stays within Phase 00 CPU/memory/latency budget
- [ ] Total series count within [METRICS_CONTRACT.md](METRICS_CONTRACT.md) budget
- [ ] No sensitive labels in scraped samples (CI regex scan)
- [ ] XMSS security counters tested (`EtheanXmssReuseDetected` on single increment)

### Evidence artifacts

- [ ] `spec/pins/phase-12.lock.toml` includes observability image digests
- [ ] `docs/lean-consensus-migration-phase-12-observability.md` summary
- [ ] Hive or CI run ID linked in evidence manifest

Failure of any gate blocks Phase 13 release qualification.

## CI job outline

```text
observability-multinode (Phase 12)
├── build ethean (2 instances) + pull peer image @ pin
├── gen-prometheus-targets.json
├── docker compose -f deploy/observability/docker-compose.lab.yml up
├── wait: all up==1 and readyz==200
├── run promtool checks + rule unit tests
├── grafana provisioning smoke (API list dashboards)
├── fault: prover hang → expect alerts
├── fault: network partition → expect alerts
├── collect evidence tarball
└── compose down -v
```

## Related documents

- [README.md](README.md) — Phase 12 scope and reading order
- [../phases/11-storage-sync-and-checkpoints.md](../phases/11-storage-sync-and-checkpoints.md) — upstream dependency
- [../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md) — exporter budget
- [../04-risks/SECURITY_GATES.md](../04-risks/SECURITY_GATES.md) — G0 observability pins
