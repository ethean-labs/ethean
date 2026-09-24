# leanMetrics: the last unrecorded series — 2026-09-24

All 63 leanMetrics series are now recorded. The three that were still empty:

- `lean_connected_peers{client}` and `lean_gossip_mesh_peers{client}`: the
  QUIC swarm now runs libp2p identify (protocol `/leanconsensus/1`, agent
  `ethean/<version>`) and keeps each peer's agent version. Peers are grouped by
  client family, the first token of the agent string (`ream/0.4.1` -> `ream`),
  or `unknown` before identify completes. Mesh membership is the union of
  gossipsub mesh peers over the subscribed block, aggregation and attestation
  topics (`crates/network/src/quic_swarm_clients.rs`).
- `lean_fork_choice_reorg_depth`: when the head moves onto a block that does
  not extend the previous head, `ChainOwner::reorg_depth` walks parents from the
  durable blobs and the fork-choice store and observes the number of blocks the
  old branch loses to the common ancestor (at least 1).

`lean_latest_known_aggregated_payloads` was completed with the post-block
proof recovery work earlier today.

## Schema label

`build_info` now reports `ethean-metrics-v4`. The bump follows the metrics
contract: the `client` label of `lean_connected_peers` changed meaning (from a
constant `unknown` to client families), `lean_gossip_mesh_peers` went from
absent to live, and three leanSpec-registry gauges joined the export. Names,
types and buckets of every previously exported series are unchanged, so v3
dashboards keep working; the label only tells operators which build they scrape.
