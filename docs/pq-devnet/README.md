# pq-devnet / operator run

Dated development notes for this topic. Index: [../README.md](../../README.md).

- [default network pq devnet 4 keep d5 ready](default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md)
- [ethean data dir run logs](ethean-data-dir-run-logs-2026-09-20.md)
- [ethean path command after build](ethean-path-command-after-build-2026-09-20.md)
- [hive quickstart cli contract](hive-quickstart-cli-contract-2026-09-24.md)
- [local pq mesh private dial](local-pq-mesh-private-dial-2026-09-20.md)
- [pq devnet 5 client run](pq-devnet-5-client-run-2026-09-19.md)
- [pq devnet 5 research refresh](pq-devnet-5-research-refresh-2026-09-19.md)
- [pq devnet operator plug in checklist](pq-devnet-operator-plug-in-checklist-2026-09-20.md)
- [registry privkey load](registry-privkey-load-2026-09-20.md)
- [start pq devnet 5 network target](start-pq-devnet-5-network-target-2026-09-19.md)
- [working client pq devnet 5 plan](working-client-pq-devnet-5-plan-2026-09-19.md)
- [wait-for-genesis-2026-09-24.md](./wait-for-genesis-2026-09-24.md) — duty loops idle until `GENESIS_TIME` instead of exiting (hive / quickstart start clients before genesis)
- [local-proving-devnet-windows-2026-09-29.md](./local-proving-devnet-windows-2026-09-29.md) — `ethean devnet-init` + `scripts/local-devnet.ps1`: N proving nodes with PROD keys on one host
- [late-join-restart-mesh-2026-09-29.md](./late-join-restart-mesh-2026-09-29.md) — checkpoint-sync joiner and data-dir restart on the proving mesh; outbox flush, gossip orphan and serve-cache fixes
- [scale-multi-aggregator-2026-09-30.md](./scale-multi-aggregator-2026-09-30.md) — several validators per node (per-index keys), subnet-scoped aggregation, cross-subnet union merges, `-ValidatorsPerNode` / `-Subnets` / `-Aggregators` runs
- [soak-memory-bounds-2026-09-30.md](./soak-memory-bounds-2026-09-30.md) — 20-minute soak, bounded serve cache / request tracker / fork-choice history / gossip ids, cheaper durable flush
- [vote-retry-and-prune-index-2026-09-30.md](./vote-retry-and-prune-index-2026-09-30.md) — deferred retry of votes rejected for a missing block or an early slot; pruning reads slots from file headers and a redb index instead of decoding
- [prover-thread-share-2026-09-30.md](./prover-thread-share-2026-09-30.md) — provers on one host share the cores (`ETHEAN_PROVER_COUNT`, `LEANVM_NUM_THREADS`); measured block proof times and the 2-aggregator finality result
