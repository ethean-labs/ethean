# leanMetrics standard metrics, schema v3 (2026-09-23)

## Why

Ethean exported only `ethean_*` names, so the shared lean client dashboards
and observatory.leanroadmap.org could not see it. leanEthereum/leanMetrics
defines 63 standard `lean_*` metrics every client exports under the same
names, types, labels and buckets.

## What changed

- `ethean-metrics` gained a `lean` module: a table of all 63 metrics, a
  thread-safe store with labeled series and histograms, and Prometheus text
  export. `/metrics` serves the `lean_*` families after the `ethean_*` ones;
  every `ethean_*` metric is unchanged. `build_info` reports
  `ethean-metrics-v3`.
- The table is checked against a vendored copy of the spec
  (`crates/metrics/testdata/leanmetrics-69f9722.md`): every documented metric,
  type, label key and bucket must match.
- Recording points in the node:
  - PQ signatures: local signing, gossip vote verification (valid / invalid,
    time), Type-1 building and verification, attestations per aggregate.
  - Block production: build time, success / failure, payloads per block,
    Type-2 merge time.
  - Fork choice / STF: block processing time, state transition phase timings
    (the transition now returns `TransitionTimings`), slots and attestations
    processed, finalizations, head / current / safe-target / justified /
    finalized slots, sync status, tick interval, signature and aggregate pools.
  - Validator: validators, aggregator flag, attestation production time,
    aggregation skips by reason.
  - Network: connected peers, connection / disconnection events with
    direction and result / reason (the swarm pump now carries both), committee
    count and subnet, gossip sizes, gossip arrival delay and position
    (block interval 0, votes interval 1, aggregates interval 2).
- Grafana provisions the upstream leanMetrics interop dashboard
  (`deploy/observability/grafana/dashboards/lean-ethereum-client-interop.json`).

## Not populated yet

- `lean_fork_choice_reorg_depth`: Ethean counts reorgs but has no fork-choice
  store to measure depth.
- `lean_latest_known_aggregated_payloads`: Ethean keeps one aggregate pool,
  reported as `lean_latest_new_aggregated_payloads`.
- `lean_gossip_mesh_peers`: the swarm does not expose gossipsub mesh size yet.
- `lean_connected_peers` uses `client="unknown"`; peer client names are not
  identified.

## Evidence

- `cargo test -p ethean-metrics` (spec conformance and exposition format).
- `cargo test --release -p ethean-node --test aggregation_flow` asserts the
  PQ signature, aggregation, block production, STF and gossip series move
  during a real vote -> aggregate -> block -> import run.
