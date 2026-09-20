# leanMetrics name map and safe_target gauge (2026-09-20)

## Goal

Align Ethean scrape surface with [leanMetrics](https://github.com/leanEthereum/leanMetrics)
without renaming the `ethean_` prefix yet (OSD-009 pin still open).

## What landed

### Metrics

- `ethean_safe_target_slot` — gauge (catalog analogue: `lean_safe_target_slot`)
- `ethean_fc_reorg_total` — counter for future head-reorg detection

Node wiring: `safe_target_slot` currently mirrors `head_slot` because
`ChainOwner` still stubs `safe_target` to the head root until a live
`ForkChoiceStore` is owned in-process. The series exists so Grafana /
leanMetrics dashboards can bind early.

### Name map (subset)

| leanMetrics | Ethean |
| --- | --- |
| `lean_head_slot` | `ethean_head_slot` |
| `lean_current_slot` | `ethean_slot_current` |
| `lean_latest_justified_slot` | `ethean_justified_slot` |
| `lean_latest_finalized_slot` | `ethean_finalized_slot` |
| `lean_safe_target_slot` | `ethean_safe_target_slot` |
| `lean_connected_peers` | `ethean_peer_count` |
| `lean_validators_count` | `ethean_validator_count` |
| `lean_is_aggregator` | `ethean_aggregator_enabled` |
| `lean_node_start_time_seconds` | `ethean_start_time_seconds` |

## Still open

- Wire real safe-target from FC store; increment `fc_reorg_total` on reorg
- PQ sig / gossip-arrival histograms when signing path is live
- Optional leanMetrics matrix column for Ethean
