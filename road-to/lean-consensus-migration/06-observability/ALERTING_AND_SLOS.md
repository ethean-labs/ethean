# Alerting and SLOs

Alert rules express operator-actionable conditions derived from [METRICS_CONTRACT.md](METRICS_CONTRACT.md). Recording rules keep PromQL cheap and consistent for [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md).

Alertmanager **routing** (PagerDuty, Slack, email) is deployment-owned. Rule **semantics and tests** live in the Ethean repo.

## SLO overview

| SLO | Measurement | Target (initial; refine in Phase 00) |
|---|---|---|
| Slot participation | missed duty rate / assigned | < 1% over 1 h per role |
| Finality liveness | time between finalized advances | < 3 epochs under normal load |
| Head freshness | head slot vs fleet max | within 2 slots |
| Proof timeliness | prover job before duty deadline | p99 meets Phase 00 budget |
| Gossip validity | validated / received | > 99% excluding known `ignored` |
| Peer connectivity | connected peers vs baseline | within 20% of profile baseline |
| Sync catch-up | sync_distance decrease rate | decreasing when syncing |
| Availability | `up` and `/readyz` | 99.9% monthly |
| Recovery | time to ready after restart | < Phase 00 p95 |

Numeric targets are placeholders until Phase 00 baselines approve limits ([../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md)).

## Recording rules

File: `deploy/observability/prometheus/rules/ethean_recording.yml`

| Record | Expression intent |
|---|---|
| `ethean:head_slot:max` | `max by (network, snapshot) (ethean_head_slot)` |
| `ethean:finality_lag_slots` | head − finalized per node |
| `ethean:finalized_rate:5m` | `rate(ethean_finalized_change_total[5m])` |
| `ethean:duty_miss_rate:5m` | misses / assigned by role, duty |
| `ethean:prover_queue_saturation` | queue_bytes / limit |
| `ethean:peer_count:min` | min connected peers per network partition |
| `ethean:sync_distance:max` | max lag across fleet |
| `ethean:xmss_leaves_remaining:min` | min leaves by role |
| `ethean:scrape_success` | `up` joined with ready gauge |

Histogram quantiles use `_bucket` aggregation:

```yaml
- record: ethean:prover_job_seconds:p99:5m
  expr: |
    histogram_quantile(0.99,
      sum by (le, network, snapshot) (rate(ethean_prover_job_seconds_bucket[5m]))
    )
```

## Alert rules

File: `deploy/observability/prometheus/rules/ethean_alerts.yml`

Severity: `critical` (immediate page), `warning` (ticket), `info` (dashboard annotation only).

### Consensus and duties

| Alert | Condition (sketch) | Severity | For |
|---|---|---|---|
| `EtheanMissedProposal` | increase missed block duties > 0 in 2 slots | critical | 0m |
| `EtheanMissedAttestation` | miss rate > SLO | warning | 10m |
| `EtheanFinalityStall` | finalized rate = 0 AND head advancing | critical | 2× epoch duration |
| `EtheanStaleHead` | head lag vs fleet max > 2 slots | warning | 5m |
| `EtheanDutyDeadlineMiss` | increase deadline_miss > threshold | warning | 5m |
| `EtheanReorgDepthHigh` | reorg depth > bucket N | warning | 0m |

### Prover and proof

| Alert | Condition | Severity | For |
|---|---|---|---|
| `EtheanProofTimeout` | increase prover timeout > 0 | critical | 0m |
| `EtheanProverQueueSaturated` | saturation > 0.9 | warning | 5m |
| `EtheanProverWorkerStale` | heartbeat age > limit | critical | 2m |
| `EtheanProofCoverageLow` | coverage ratio < SLO | warning | 15m |
| `EtheanProverOOMRisk` | peak_bytes near limit | warning | 5m |

### P2P and req-resp

| Alert | Condition | Severity | For |
|---|---|---|---|
| `EtheanPeerLoss` | peer count drop > 50% vs 1 h baseline | warning | 5m |
| `EtheanPartitionSuspect` | asymmetric peer counts across nodes | critical | 5m |
| `EtheanReqRespErrorRate` | error rate > SLO | warning | 10m |
| `EtheanGossipRejectBurst` | reject rate spike | warning | 5m |

### Storage, sync, recovery

| Alert | Condition | Severity | For |
|---|---|---|---|
| `EtheanDiskPressure` | disk free < threshold | critical | 5m |
| `EtheanStorageCorruption` | corruption counter increase | critical | 0m |
| `EtheanSyncStalled` | syncing AND distance not decreasing | warning | 15m |
| `EtheanCheckpointStale` | checkpoint age > limit | warning | 10m |
| `EtheanRestartRecoverySlow` | ready false > p95 after restart | warning | 10m |

### XMSS security

| Alert | Condition | Severity | For |
|---|---|---|---|
| `EtheanXmssReuseDetected` | reuse counter increase | critical | 0m |
| `EtheanXmssRollbackDetected` | rollback counter increase | critical | 0m |
| `EtheanXmssLeafLow` | leaves remaining < threshold | critical | 0m |
| `EtheanXmssLeafLifetime` | projected exhaustion < 7 d | warning | 1h |

### Observability stack

| Alert | Condition | Severity | For |
|---|---|---|---|
| `EtheanTargetDown` | up == 0 | critical | 2× scrape interval |
| `EtheanNotReady` | ready == 0 while process up | warning | 5m |
| `EtheanScrapeSlow` | scrape duration > timeout × 0.8 | warning | 10m |
| `EtheanHighCardinality` | sample count > budget | warning | 5m |

Example rule fragment:

```yaml
groups:
  - name: ethean_consensus
    rules:
      - alert: EtheanFinalityStall
        expr: |
          (ethean:finalized_rate:5m == 0)
          and (changes(ethean_head_slot[5m]) > 0)
        for: 8m
        labels:
          severity: critical
        annotations:
          summary: Finality not advancing while head moves
          description: |
            Network {{ $labels.network }} snapshot {{ $labels.snapshot }}.
            Check consensus, network partition, and supermajority availability.
```

## CI: parse, lint, and test

### Config lint

```bash
promtool check config deploy/observability/prometheus/prometheus.yml
promtool check rules deploy/observability/prometheus/rules/*.yml
```

Runs in CI with the **same pinned Prometheus image** as production ([PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md)).

### Rule unit tests

Directory: `deploy/observability/prometheus/rules/test/`

```yaml
# deploy/observability/prometheus/rules/test/finality_stall_test.yml
rule_files:
  - ../ethean_alerts.yml
  - ../ethean_recording.yml

evaluation_interval: 4s

tests:
  - interval: 4s
    input_series:
      - series: 'ethean_finalized_change_total{node="ethean-0",network="test"}'
        values: '0 0 0 0 0'
      - series: 'ethean_head_slot{node="ethean-0",network="test"}'
        values: '100 101 102 103 104'
    alert_rule_test:
      - eval_time: 10m
        alertname: EtheanFinalityStall
        exp_alerts:
          - exp_labels:
              node: ethean-0
              network: test
            exp_annotations:
              summary: Finality not advancing while head moves
```

Required test coverage before Phase 12 exit:

- Finality stall, stale head, missed proposal
- Proof timeout, queue saturation
- Peer loss, req-resp error burst
- Disk pressure, restart recovery slow
- XMSS reuse (must fire on single increment)
- Target down

### Fixture metric smoke tests

1. Load golden `/metrics` from `ethean-metrics` fixture into Prometheus test harness OR use `promtool test rules` input_series only.
2. Assert every alert name has at least one test case.
3. Assert no alert query references forbidden labels.
4. Run `scripts/alert-cardinality-check.sh` to ensure alert label set stays bounded.

## Synthetic fault validation (Phase 12)

Manual or automated fault injection in lab profile; alerts must fire and clear:

| Fault | Expected alerts |
|---|---|
| Stop proposer mid-slot | `EtheanMissedProposal`, possible `EtheanDutyDeadlineMiss` |
| Block finality path (test hook) | `EtheanFinalityStall` |
| Hang prover worker | `EtheanProverWorkerStale`, `EtheanProofTimeout` |
| Isolate node network | `EtheanPeerLoss`, possible `EtheanPartitionSuspect` |
| Fill disk | `EtheanDiskPressure` |
| Kill and restart node | `EtheanTargetDown`, then `EtheanRestartRecoverySlow` if slow |
| Inject xmss reuse counter | `EtheanXmssReuseDetected` |

Results logged in Phase 12 evidence manifest.

## Alert hygiene

- Every alert has `summary`, `description`, and `runbook_url` annotation pointing to ops doc.
- No alert fires on single-scrape blips unless security-critical (XMSS reuse).
- `for:` duration ≥ 1 scrape interval except security counters.
- Alerts include `network`, `snapshot`, `node`, `client` where applicable; never validator index or peer ID.
- Inhibit rules: `EtheanTargetDown` inhibits other alerts for same `instance`.

## Related documents

- [METRICS_CONTRACT.md](METRICS_CONTRACT.md)
- [GRAFANA_DASHBOARDS.md](GRAFANA_DASHBOARDS.md)
- [MULTINODE_ANALYSIS.md](MULTINODE_ANALYSIS.md) — multi-node fault scenarios
- [../04-risks/SECURITY_GATES.md](../04-risks/SECURITY_GATES.md) — G0 pin requirements
