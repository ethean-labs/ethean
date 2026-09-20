# Richer Grafana monitors from live ethean_ metrics (2026-09-20)

## Client already exposes (and now more)

Scraped from `http://127.0.0.1:9100/metrics` (`ethean-metrics-v2`):

| Metric | Panel use |
| --- | --- |
| `ethean_start_time_seconds` | Latest start time (green; Grafana uses `* 1000`) |
| `ethean_validator_count` | Validators |
| `ethean_aggregator_enabled` | Aggregator on/off |
| `ethean_local_finality_enabled` | Local finality on/off |
| `ethean_peer_count` | Peers (QuicSwarm connections) |
| `ethean_bootnode_count` | Configured dial targets |
| `ethean_ready` (+ `ready_*` gates) | Ready / subsystem health |
| `ethean_seconds_per_slot` / `ethean_genesis_time_seconds` | Clock identity |
| `ethean_finalized_slot` / `justified_slot` / `head_slot` / `slot_current` | Slot stats + step charts |
| `ethean_finality_lag_slots` / `justification_lag_slots` / `sync_lag_slots` | Lag row |
| `ethean_duty_suppressed_total` / `ethean_prover_timeout_total` | Ops counters |
| `ethean_build_info` | Node info table |

See also [grafana-no-data-and-1970-fix-2026-09-20.md](./grafana-no-data-and-1970-fix-2026-09-20.md).

## Grafana (Ethean-branded, Ream-layout inspired)

1. **Ethean Lean Clients Dashboard** (`ethean-lean-clients`) — overview like the
   Lean mesh board: green “good” stats, four slot step charts, genesis + build info.
2. **Ethean Node Health** (`ethean-node-health`) — readiness gates, peers, lags,
   duty/prover counters, Prometheus `up`.

No peer client names in titles or queries. Folder: Grafana → **Ethean**.

## Operator

```text
ethean start --until-signal --network pq-devnet-4 --data-dir ./ethean-data --metrics
```

Open http://localhost:3000 (after Docker compose / `--metrics`). Reload
provisioned dashboards if the stack was already running:

```text
docker compose -f deploy/observability/docker-compose.yml up -d
```
