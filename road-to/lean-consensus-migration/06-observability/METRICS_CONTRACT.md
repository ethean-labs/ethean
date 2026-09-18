# Metrics Contract

Contract owner: `ethean-metrics` crate. Phase 12 implements this document; CI enforces it before merge.

## Naming and compatibility

Use the pinned **leanMetrics** name and semantics when available. Record that version in build information. Metrics absent from leanMetrics use the **`ethean_`** prefix. Names include base units (`_seconds`, `_bytes`) and counters end in `_total`. A semantic or label change requires a new metric name or `metrics_schema` version bump; dashboards declare a supported compatibility window.

Every build exposes:

| Metric | Type | Labels (bounded) |
|---|---|---|
| `ethean_build_info` | Info/gauge = 1 | `version`, `git_commit`, `rustc`, `protocol_pin`, `crypto_pin`, `metrics_schema`, `leanmetrics_pin` (empty if N/A) |
| `ethean_node_role_info` | Info/gauge = 1 | `role`, `network` |
| Process/runtime collectors | standard | from maintained `process_*` / runtime collectors only |

Build labels are bounded release values. They must not accept arbitrary runtime text.

## Exporter lifecycle at node start

Implemented in `ethean-metrics/src/export/` and wired from `ethean` runtime startup ([../03-architecture/TARGET_WORKSPACE.md](../03-architecture/TARGET_WORKSPACE.md)).

### Configuration surface

| Setting | Default | Notes |
|---|---|---|
| `metrics.enabled` | `true` in production profiles | `false` only in explicit local-dev profile |
| `metrics.listen_addr` | `127.0.0.1:9090` | Separate from RPC/API listener |
| `metrics.unsafe_expose` | `false` | Must be `true` to bind non-loopback |
| `metrics.tls.*` | disabled | Required for non-loopback without reverse proxy |
| `metrics.scrape_timeout_budget` | Phase 00 derived | Must be `< scrape_interval` and `< slot reserve |
| `metrics.shutdown_grace_seconds` | Phase 00 derived | Max wait for in-flight scrape |

### Startup sequence

1. **Registry init** — register all descriptors; duplicate name/type/label-set is fatal.
2. **Cardinality guard** — reject registrations whose label cardinality product exceeds per-metric budget (see § Cardinality budgets).
3. **Bind** — listen on configured address; failure is fatal when metrics are required.
4. **Serve** — expose `/metrics`, `/healthz`, `/readyz` on the metrics listener only.
5. **Readiness gating** — `ethean_ready{component="*"}` gauges remain `0` until each subsystem completes init (see § Lifecycle and time).
6. **Node ready** — `/readyz` returns 200 only when all required components for the configured `role` are ready.

### Readiness components by role

| Component | Required for | Ready when |
|---|---|---|
| `storage` | all roles | recovery complete, schema validated |
| `crypto` | all roles | suite pin validated |
| `signer` | validator, aggregator | XMSS state loaded, no duplicate instance |
| `consensus` | all roles | fork-choice initialized, clock synced within bound |
| `p2p` | all roles | QUIC listener up, identity handshake complete |
| `prover` | aggregator (if enabled) | worker pool started, queue admitted |
| `node` | all roles | all role-required components above are ready |

### Scrape behavior

- Prometheus `scrape_timeout` is configured in [PROMETHEUS_TOPOLOGY.md](PROMETHEUS_TOPOLOGY.md); exporter must render full exposition within that budget under Phase 00 load fixture.
- Slow scrapes increment `ethean_exporter_scrape_slow_total`; they must not block consensus hot paths (collection runs on dedicated task with deadline).
- Concurrent scrapes are capped; excess connections receive `503` with `ethean_exporter_scrape_rejected_total`.

### Graceful shutdown

Triggered by node shutdown hook ([`ethean` runtime](../03-architecture/TARGET_WORKSPACE.md)):

1. Set all `ethean_ready` to `0`; `/readyz` → 503.
2. Stop accepting new HTTP connections on metrics listener.
3. Wait up to `shutdown_grace_seconds` for in-flight `/metrics` scrape.
4. Emit final counter values (no reset).
5. Close listener; join exporter task before process exit.

Bind errors, auth misconfiguration (when required), and registry init failure are **startup fatal**. Metrics must never silently disable in production profiles.

### Authentication and bind policy

| Bind | Auth requirement |
|---|---|
| Loopback (`127.0.0.1`, `::1`) | None (default safe) |
| Private RFC1918 / ULA | TLS or mTLS, or authenticated reverse proxy |
| Any routable / public | Forbidden without deployment waiver + mandatory mTLS |

Scrape credentials never appear in metric labels, HELP strings, or error responses.

## Label policy

### Allowed label dimensions (closed enums)

`result`, `reason`, `stage`, `topic`, `message_type`, `direction`, `role`, `queue`, `operation`, `backend`, `source_class`, `client_family`, `duty`, `proof_class`, `sync_mode`, `checkpoint_class`.

Unknown enum values map to `other` at record time. Raw runtime strings are never labels.

### Forbidden labels (non-exhaustive)

Validator index or ID, public/private key material, signature bytes, XMSS key or leaf index, peer ID, IP address, multiaddress, block/state root, message ID, request ID, filesystem path, error string, panic text, witness/proof bytes or hash, URL, user-supplied free text.

Per-peer or per-validator diagnostics belong in sampled/redacted logs or short-lived traces, never time-series labels.

## Cardinality and sensitive-data budgets

Acceptance criteria for Phase 12 exit:

| Budget | Limit source | Enforcement |
|---|---|---|
| Total active series per process | Phase 00 fixture (target: ≤ 5 000 under max devnet topology) | CI load fixture + `prometheus` sample_limit |
| Series per metric family | ≤ 200 combinations in fixture | Registration-time product check |
| New label approval | Architecture review + CI diff | Required for any new label name |
| Scrape body size | Phase 00 p99 × 1.5 | Prometheus `body_size_limit` |
| Sensitive pattern scan | Zero matches in golden exposition | CI regex on `/metrics` fixture |
| Label value entropy | No label with > 20 distinct values in 1 h lab soak | Soak test + lint |

Violation of any budget blocks Phase 12 exit until the metric is redesigned or aggregated upstream of export.

## Core metric families

Histogram buckets are derived from Phase 00 distributions and the four-second slot envelope. Shared bucket boundaries across nodes enable valid fleet aggregation. Bucket changes require `metrics_schema` bump.

### Lifecycle, slot, and interval

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_ready` | Gauge | `component` | 1 = ready, 0 = not ready |
| `ethean_clock_offset_seconds` | Gauge | — | NTP/chrony offset vs system clock |
| `ethean_slot_current` | Gauge | — | Current slot number |
| `ethean_epoch_current` | Gauge | — | Current epoch |
| `ethean_slot_phase_seconds` | Histogram | `stage` | Per-stage duration within slot |
| `ethean_slot_deadline_miss_total` | Counter | `stage`, `role` | Missed internal deadline |
| `ethean_interval_seconds` | Gauge | — | Configured slot duration (sanity check, expect ~4) |

`stage` enum: `receive`, `decode`, `verify`, `transition`, `persist`, `publish`, `duty_prepare`, `sign`, `prove`, `aggregate`.

### Head, justified, finalized, safe-target

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_head_slot` | Gauge | — | Head block slot |
| `ethean_head_epoch` | Gauge | — | Head block epoch |
| `ethean_justified_slot` | Gauge | — | Latest justified checkpoint slot |
| `ethean_justified_epoch` | Gauge | — | Latest justified checkpoint epoch |
| `ethean_finalized_slot` | Gauge | — | Latest finalized checkpoint slot |
| `ethean_finalized_epoch` | Gauge | — | Latest finalized checkpoint epoch |
| `ethean_safe_target_slot` | Gauge | — | Safe-target / weak-subjectivity horizon slot |
| `ethean_head_change_total` | Counter | `reason` | Head updates |
| `ethean_finalized_change_total` | Counter | `reason` | Finality advances |
| `ethean_finality_lag_slots` | Gauge | — | `head_slot - finalized_slot` |

`reason` is bounded: `new_block`, `reorg`, `sync`, `checkpoint_import`, `startup`, `other`. **No roots in labels.**

### Reorg and finality

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_reorg_total` | Counter | `reason` | Chain reorganizations |
| `ethean_reorg_depth_slots` | Histogram | — | Depth of each reorg |
| `ethean_finality_stall_seconds` | Gauge | — | Time since last finalized advance |
| `ethean_blocks_processed_total` | Counter | `result`, `reason` | Block processing outcomes |
| `ethean_attestations_processed_total` | Counter | `result`, `reason` | Attestation processing outcomes |

### Proposer, attester, aggregator duties

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_duty_assigned_total` | Counter | `role`, `duty` | Duties scheduled |
| `ethean_duty_started_total` | Counter | `role`, `duty` | Duties begun |
| `ethean_duty_completed_total` | Counter | `role`, `duty`, `result` | Terminal outcomes |
| `ethean_duty_missed_total` | Counter | `role`, `duty`, `reason` | Missed duties |
| `ethean_duty_cancelled_total` | Counter | `role`, `duty`, `reason` | Cancelled duties |
| `ethean_duty_published_total` | Counter | `role`, `duty`, `result` | Gossip/API publish |
| `ethean_duty_milestone_seconds` | Histogram | `role`, `duty`, `milestone` | Time from slot start |

`role`: `proposer`, `attester`, `aggregator`. `duty`: `block`, `attestation`, `aggregate`, `sync_committee` (if applicable). `milestone`: `prepare`, `sign`, `prove`, `publish`.

### XMSS, Type-1, Type-2 crypto durations and errors

Map to leanMetrics names when pinned; otherwise:

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_xmss_leaves_remaining` | Gauge | `role` | Aggregated remaining leaves (never per-key) |
| `ethean_xmss_leaf_reserved_total` | Counter | `role` | Leaf reservations |
| `ethean_xmss_leaf_burned_total` | Counter | `role`, `reason` | Burned leaves |
| `ethean_xmss_leaf_exhaustion_total` | Counter | `role` | Exhaustion events |
| `ethean_xmss_rollback_detected_total` | Counter | — | **Security incident** |
| `ethean_xmss_reuse_detected_total` | Counter | — | **Security incident** |
| `ethean_crypto_operation_seconds` | Histogram | `operation`, `result` | Hash/sign/verify/aggregate |
| `ethean_crypto_operation_total` | Counter | `operation`, `result`, `reason` | Terminal crypto outcomes |

`operation`: `poseidon_hash`, `xmss_sign`, `xmss_verify`, `type1_sign`, `type1_verify`, `type2_sign`, `type2_verify`, `aggregate_verify`, `proof_verify`.

### Proof coverage, size, queue, deadline (leanVM / prover)

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_proof_coverage_ratio` | Gauge | `proof_class` | Attestations/blocks with valid proof vs expected |
| `ethean_proof_input_bytes` | Histogram | `proof_class` | Witness/input size |
| `ethean_proof_output_bytes` | Histogram | `proof_class` | Proof output size |
| `ethean_prover_jobs_total` | Counter | `result` | submitted/admitted/rejected/completed/failed/timeout/cancelled/killed |
| `ethean_prover_queue_items` | Gauge | `queue` | Queued jobs |
| `ethean_prover_queue_bytes` | Gauge | `queue` | Queued witness bytes |
| `ethean_prover_active_workers` | Gauge | — | Running workers |
| `ethean_prover_worker_heartbeat_age_seconds` | Gauge | — | Oldest worker heartbeat |
| `ethean_prover_job_seconds` | Histogram | `result` | Wall time per job |
| `ethean_prover_job_cpu_seconds` | Histogram | `result` | CPU time per job |
| `ethean_prover_job_peak_bytes` | Histogram | `kind` | Peak RSS / temp disk |
| `ethean_prover_deadline_miss_total` | Counter | `reason` | Jobs missing duty deadline |

### Gossip topic, result, size, latency

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_gossip_received_total` | Counter | `topic`, `message_type` | Inbound messages |
| `ethean_gossip_validated_total` | Counter | `topic`, `result`, `reason` | Validation outcomes |
| `ethean_gossip_published_total` | Counter | `topic`, `message_type`, `result` | Outbound publish |
| `ethean_gossip_propagation_seconds` | Histogram | `topic` | Slot-relative propagation delay |
| `ethean_gossip_validation_seconds` | Histogram | `topic`, `stage` | Validation pipeline |
| `ethean_gossip_message_bytes` | Histogram | `topic`, `direction` | Message size |
| `ethean_gossip_queue_dropped_total` | Counter | `queue`, `reason` | Dropped due to pressure |

### Peer, QUIC, req-resp

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_peers_connected` | Gauge | `direction`, `client_family` | Connected peer count |
| `ethean_peer_connection_total` | Counter | `direction`, `result`, `reason` | Connection attempts |
| `ethean_peer_disconnect_total` | Counter | `direction`, `reason` | Disconnects |
| `ethean_quic_rtt_seconds` | Histogram | `direction` | RTT samples (aggregated, not per-peer) |
| `ethean_quic_loss_ratio` | Gauge | `direction` | Aggregated loss estimate |
| `ethean_reqresp_request_total` | Counter | `protocol`, `result`, `reason` | Req-resp outcomes |
| `ethean_reqresp_duration_seconds` | Histogram | `protocol`, `result` | Round-trip time |
| `ethean_reqresp_response_bytes` | Histogram | `protocol`, `direction` | Payload size |
| `ethean_discovery_round_seconds` | Histogram | `result` | Discovery cycle duration |

### Sync distance and checkpoint

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_sync_state` | Gauge | `sync_mode` | Enum encoded as small integer + info metric |
| `ethean_sync_distance_slots` | Gauge | — | Slots behind network head |
| `ethean_sync_objects_imported_total` | Counter | `object`, `result` | Imported blocks/states/checkpoints |
| `ethean_sync_request_total` | Counter | `object`, `result`, `reason` | Sync req-resp |
| `ethean_checkpoint_import_total` | Counter | `source_class`, `result` | Checkpoint operations |
| `ethean_checkpoint_age_seconds` | Gauge | `checkpoint_class` | Age of active checkpoint |

### Storage commit, recovery, pruning

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_storage_operation_seconds` | Histogram | `operation`, `backend`, `result` | DB operations |
| `ethean_storage_batch_size` | Histogram | `operation` | Records per batch |
| `ethean_storage_commit_total` | Counter | `result`, `reason` | Canonical commits |
| `ethean_storage_recovery_total` | Counter | `result`, `reason` | Recovery attempts |
| `ethean_storage_corruption_total` | Counter | `reason` | Detected corruption |
| `ethean_storage_migration_phase` | Gauge | — | Current migration phase |
| `ethean_storage_bytes` | Gauge | `kind` | logical/disk/WAL |
| `ethean_storage_prune_total` | Counter | `result` | Pruning operations |
| `ethean_storage_open_files` | Gauge | — | FD pressure indicator |

### Host resources (CPU, RAM, thread, FD)

Prefer standard process collectors; Ethean-specific supplements:

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_runtime_threads` | Gauge | `pool` | Active threads by pool |
| `ethean_runtime_task_queue_items` | Gauge | `pool` | Internal task queues |
| `ethean_process_cpu_seconds_total` | Counter | — | If not covered by `process_*` |
| `ethean_process_resident_memory_bytes` | Gauge | — | RSS |
| `ethean_process_open_fds` | Gauge | — | Open file descriptors |

### Exporter self-metrics

| Metric | Type | Labels | Semantics |
|---|---|---|---|
| `ethean_exporter_scrape_total` | Counter | `result` | Scrape requests |
| `ethean_exporter_scrape_duration_seconds` | Histogram | — | Render time |
| `ethean_exporter_scrape_bytes` | Histogram | — | Response size |
| `ethean_exporter_scrape_in_flight` | Gauge | — | Active scrapes |
| `ethean_exporter_scrape_rejected_total` | Counter | `reason` | Rejected connections |
| `ethean_exporter_auth_failure_total` | Counter | — | Auth failures (no client id label) |
| `ethean_exporter_registry_collect_seconds` | Histogram | — | Collector pass duration |
| `ethean_exporter_series_count` | Gauge | — | Active series |
| `ethean_exporter_last_success_timestamp_seconds` | Gauge | — | Last successful render |

Do not label exporter metrics by scraper IP or user agent.

## Counter and reset semantics

Counters are monotonic for one process lifetime. Dashboards use `rate`/`increase` and tolerate restart via `changes()` or counter reset detection. Gauges document whether `0` means healthy zero, unknown, or absent. Unknown values omit the sample and expose readiness/error metrics rather than misleading zeros.

## Instrumentation requirements

- Timers cover queue wait separately from service time.
- Result counters increment exactly once at terminal state.
- Cancellation and rejection are distinct from failure.
- Hot-path instrumentation avoids allocation and blocking.
- Registry collection has a benchmark-derived deadline ([../04-risks/PERFORMANCE_BUDGETS.md](../04-risks/PERFORMANCE_BUDGETS.md)).
- Metric registration failure is fatal at startup.

A nonzero `ethean_xmss_reuse_detected_total` or `ethean_xmss_rollback_detected_total` is a security incident. Metrics supplement the durable signing audit; they do not replace it.

## Contract tests

CI in `ethean-metrics` and Phase 12 integration:

1. Parse golden `/metrics` exposition fixture.
2. Verify HELP/TYPE/unit suffixes and name uniqueness.
3. Assert bounded labels and absence of forbidden patterns.
4. Verify counter monotonicity and gauge semantics across simulated restart.
5. Compatibility check against provisioned recording rules, alert rules, and dashboards.
6. Load fixture at maximum bounded label combinations; enforce total-series budget.

Golden fixtures live under `crates/ethean-metrics/tests/fixtures/` (planned in Phase 12).
