# docs

Design notes, sprint write-ups, and working plans for the client. Root README should link anything here that a new contributor actually needs.

Root README companion guides (short landing page + detail): [readme/README.md](./readme/README.md)
([architecture](./readme/architecture.md), [installation](./readme/installation.md),
[usage](./readme/usage.md), [api](./readme/api.md), [development](./readme/development.md),
[testing](./readme/testing.md)). Session notes:
[readme-companion-guides-2026-09-20.md](./readme-companion-guides-2026-09-20.md),
[readme-overview-architecture-2026-09-20.md](./readme-overview-architecture-2026-09-20.md),
[readme-overview-expanded-2026-09-20.md](./readme-overview-expanded-2026-09-20.md),
[readme-header-nav-2026-09-20.md](./readme-header-nav-2026-09-20.md),
[readme-peer-wording-2026-09-20.md](./readme-peer-wording-2026-09-20.md),
[readme-version-footer-2026-09-20.md](./readme-version-footer-2026-09-20.md).

Contributor guide (root): [../CONTRIBUTING.md](../CONTRIBUTING.md). Session note: [contributing-guide-2026-09-20.md](./contributing-guide-2026-09-20.md).

Code of conduct (root): [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md). Session note: [code-of-conduct-2026-09-20.md](./code-of-conduct-2026-09-20.md).

Organization profile README draft (`gitreadme.md`): [org-profile-readme-gitreadme-2026-09-20.md](./org-profile-readme-gitreadme-2026-09-20.md).

Lean R&D capture (public pointer only): [leanroadmap-local-notes.md](./leanroadmap-local-notes.md). The detailed extracts live in the local `bazalinacaklar/` folder.

Seven-client source research: [lean-peer-client-research-library-2026-09-19.md](./lean-peer-client-research-library-2026-09-19.md). The full per-client audits and planning checklist remain in the local research folder.

Code layout: [source-file-size-limit.md](./source-file-size-limit.md) (300-line cap, split by module).

Peer clients (reference, not a clone): [peer-reference-clients.md](./peer-reference-clients.md).

Language: [english.md](./english.md) (all in-repo text in English).

Git: [commit-after-each-file.md](./commit-after-each-file.md) (English commit after each file, even mid-prompt).

Version: [versioning.md](./versioning.md) (root `VERSION`, patch bumps, `0.1.99` → `0.2.0`).
[VERSION file landed](./ethean-version-file-2026-09-20.md).

No AI / Cursor attribution: [no-ai-git-attribution-2026-09-19.md](./no-ai-git-attribution-2026-09-19.md) (no Cursor co-author trailers or emails in commits/PRs).

Unpublished Cursor trailer scrub: [scrub-unpublished-cursor-trailers-2026-09-19.md](./scrub-unpublished-cursor-trailers-2026-09-19.md).

Full master history trailer scrub: [full-cursor-trailer-scrub-2026-09-20.md](./full-cursor-trailer-scrub-2026-09-20.md) (local master clean; remote needs force-with-lease if approved).

Active migration planning library: [../road-to/lean-consensus-migration/README.md](../road-to/lean-consensus-migration/README.md) (charter, baseline, protocol pins, architecture, risks, retirement, observability, phases 00–13).

Planning session summaries:

- [Baseline planning](./lean-consensus-migration-baseline-planning-2026-09-19.md)
- [Protocol and retirement docs](./lean-consensus-migration-protocol-retirement-docs-2026-09-19.md)
- [Observability planning](./lean-consensus-migration-observability-planning-2026-09-19.md)
- [Phases 03–06](./lean-consensus-migration-phases-03-06-planning-2026-09-19.md)
- [Full library completion](./lean-consensus-migration-library-complete-2026-09-19.md)
- [Planning retirement closeout](./lean-consensus-migration-retire-legacy-2026-09-19.md)
- [Phase 00 compatibility snapshot](./lean-consensus-migration-phase-00-2026-09-19.md)
- [Phase 01 identity cleanup](./lean-consensus-migration-phase-01-identity-2026-09-19.md)
- [Phase 02 workspace / primitives / profile](./lean-consensus-migration-phase-02-workspace-2026-09-19.md)
- [Phase 03 canonical SSZ and Lean types](./lean-consensus-migration-phase-03-ssz-types-2026-09-19.md)
- [Full State SSZ encode/decode](./state-ssz-encode-decode-complete-2026-09-20.md)
- [Phase 04 genesis and 4s slot clock](./lean-consensus-migration-phase-04-genesis-clock-2026-09-19.md)
- [Phase 05 Lean state transition](./lean-consensus-migration-phase-05-state-transition-2026-09-19.md)
- [Phase 06 Lean fork choice (3SF-mini / lstar)](./lean-consensus-migration-phase-06-fork-choice-2026-09-19.md)
- [Phase 07 XMSS signer safety](./lean-consensus-migration-phase-07-xmss-signer-2026-09-19.md)
- [Phase 08 leanVM aggregation](./lean-consensus-migration-phase-08-leanvm-aggregation-2026-09-19.md)
- [Phase 09 validator and node duties](./lean-consensus-migration-phase-09-validator-duties-2026-09-19.md)
- [Phase 10 QUIC gossip and req/resp](./lean-consensus-migration-phase-10-quic-gossip-2026-09-19.md)
- [Phase 11 storage sync and checkpoints](./lean-consensus-migration-phase-11-storage-sync-2026-09-19.md)
- [Phase 12 API and observability](./lean-consensus-migration-phase-12-api-observability-2026-09-19.md)
- [Phase 13 security performance and release](./lean-consensus-migration-phase-13-release-2026-09-19.md)
- [ethean-node Lean shell compile (BLS purge)](./ethean-node-lean-shell-compile-2026-09-19.md)
- [ethean-node docs soft-scan cleanup](./ethean-node-docs-soft-scan-2026-09-19.md)
- [Duty loop and RocksDB gate](./duty-loop-and-rocksdb-gate-2026-09-19.md)
- [Observability, wall tick, hard BLS scan](./observability-wall-tick-hard-scan-2026-09-19.md)
- [Wall-clock run mode and QUIC gate](./wall-clock-run-and-quic-gate-2026-09-19.md)
- [Open gates closeout (until-signal, UDP, RocksDB, FFI)](./open-gates-closeout-2026-09-19.md)
- [External gates: QuicSwarm, RocksDB INCLUDE, leanSig vendor](./external-gates-quic-rocksdb-leansig-2026-09-19.md)
- [Durable QuicSwarm, leanVM stub, leanSig vendor patch](./durable-quic-leanvm-leansig-vendor-2026-09-19.md)
- [B1 leanSig vendor backend compile path](./b1-leansig-vendor-backend-compile-2026-09-20.md)
- [B2 leanVM IPC live probe (protocol_ready)](./b2-leanvm-ipc-live-probe-2026-09-20.md)
- [pq-devnet operator plug-in checklist (A2/A3/leanVM/leansig/Hive)](./pq-devnet-operator-plug-in-checklist-2026-09-20.md)
- [Aggregator prove routes to leanVM IPC](./aggregator-prove-ipc-route-2026-09-20.md)
- [Status sync prefers blocks-by-range on deep lag](./status-sync-prefer-range-on-deep-lag-2026-09-20.md)
- [Hive / leanSpec fixture consumer scaffold](./hive-leanspec-fixture-consumer-2026-09-20.md)
- [leanSpec fork-choice rejection runner](./leanspec-fc-rejection-runner-2026-09-20.md)
- [leanSpec FC ticks, imports, more rejections](./leanspec-fc-tick-import-rejections-2026-09-20.md)
- [leanSpec FC attestation steps](./leanspec-fc-attestation-steps-2026-09-20.md)
- [leanSpec FC block body attestations](./leanspec-fc-block-body-attestations-2026-09-20.md)
- [leanSpec FC gossipAggregatedAttestation](./leanspec-fc-gossip-aggregated-2026-09-20.md)
- [leanSpec FC finality / reorg / LMD suites](./leanspec-fc-finality-reorg-lmd-2026-09-20.md)
- [leanSpec FC safe-target + reorg_total](./leanspec-fc-safe-target-reorg-total-2026-09-20.md)
- [leanSpec FC prune votes not blocks](./leanspec-fc-prune-votes-not-blocks-2026-09-20.md)
- [leanSpec FC extra suite expansion](./leanspec-fc-extra-suite-2026-09-20.md)
- [leanSpec FC tick safe-target snapshot gate](./leanspec-fc-tick-safe-snapshot-gate-2026-09-20.md)
- [leanSpec FC finalized_safety empty-body gate](./leanspec-fc-finalized-safety-empty-body-gate-2026-09-20.md)
- [leanEthereum official repos plan](./lean-ethereum-official-repos-plan-2026-09-20.md)
- [leanEthereum/pm indexed](./lean-ethereum-pm-indexed-2026-09-20.md)
- [FC MAX_ATTESTATIONS_DATA + D4 log_inv_rate](./fc-max-attestations-d4-log-inv-rate-2026-09-20.md)
- [leanMetrics safe_target + name map](./leanmetrics-safe-target-name-map-2026-09-20.md)
- [leanSpec FC payload LMD weights](./leanspec-fc-payload-lmd-weights-2026-09-20.md)
- [leanSpec FC wall-clock tick + justification](./leanspec-fc-wall-clock-tick-justification-2026-09-20.md)
- [leanSpec FC checks + storeSnapshot](./leanspec-fc-checks-snapshot-2026-09-20.md)
- [leanSpec STF runner scaffold](./leanspec-stf-runner-2026-09-20.md)
- [leanSpec STF full suite green](./leanspec-stf-full-suite-green-2026-09-20.md)
- [leanSpec FC blockWeights snapshot](./leanspec-fc-block-weights-snapshot-2026-09-20.md)
- [leanSpec FC aggregated payload pools](./leanspec-fc-payload-pools-snapshot-2026-09-20.md)
- [Hive client Docker scaffold](./hive-client-docker-scaffold-2026-09-20.md)
- [Lean HTTP API on :5052](./lean-http-api-5052-2026-09-20.md)
- [Fixed QUIC listen port :9000](./quic-fixed-listen-port-2026-09-20.md)
- [Hive config.yaml / validators.yaml consume](./hive-lean-config-consume-2026-09-20.md)
- [Registry privkey load](./registry-privkey-load-2026-09-20.md)
- [Local attester duty wiring](./local-attester-duty-2026-09-20.md)
- [Owned-index proposer gate](./owned-index-proposer-gate-2026-09-20.md)
- [Gossipsub mesh on QuicSwarm](./gossipsub-quic-mesh-2026-09-19.md)
- [Gossip ingest and peer score feedback](./gossip-ingest-peer-score-2026-09-19.md)
- [SSZ gossip decode into ImportBlock](./ssz-gossip-import-block-2026-09-19.md)
- [Gossip attestation SSZ and structural STF](./gossip-attestation-stf-2026-09-19.md)
- [Gossip attestation pool and verified SignedBlock STF](./gossip-attestation-pool-verified-stf-2026-09-19.md)
- [Pool-backed block body selection](./pool-backed-block-body-2026-09-19.md)
- [PlanTransition from pool on duty ticks](./plan-transition-pool-duty-2026-09-19.md)
- [SignedBlock assemble and gossip publish](./signed-block-gossip-publish-2026-09-19.md)
- [Auto-flush pending block gossip on wall ticks](./auto-flush-block-gossip-2026-09-19.md)
- [Local proposer signing on duty ticks](./local-proposer-signing-2026-09-19.md)
- [Type-2 prove attach on proposal path](./type2-prove-attach-2026-09-19.md)
- [leanVM gate and leanSig proposer features](./leanvm-gate-leansig-proposer-2026-09-19.md)
- [Type-2 block-root binding and leanSig pin checks](./type2-block-root-leansig-pin-2026-09-19.md)
- [Proposer binding verify before gossip](./proposer-binding-verify-before-gossip-2026-09-19.md)
- [leanVM statement wire and process-IPC gate](./leanvm-statement-wire-ipc-gate-2026-09-19.md)
- [Type-2 proposer Sidecar policy](./type2-proposer-sidecar-policy-2026-09-19.md)
- [Remote proposer sidecar verify on gossip](./remote-proposer-sidecar-verify-2026-09-19.md)
- [Working-client pq-devnet-5 plan](./working-client-pq-devnet-5-plan-2026-09-19.md)
- [pq-devnet-5 research refresh (status + links)](./pq-devnet-5-research-refresh-2026-09-19.md)
- [Start pq-devnet-5 network target](./start-pq-devnet-5-network-target-2026-09-19.md)
- [Default network pq-devnet-4 (keep D5 ready)](./default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md)
- [Ream empty lean_peers.yaml (no public D4 bootnodes)](./ream-empty-lean-peers-bootnodes-2026-09-20.md)
- [How Ream connects without public bootnodes](./how-ream-connects-without-public-bootnodes-2026-09-20.md)
- [How Ream, ethlambda, and Zeam run pq-devnets](./peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md)
- [Peer fixed genesis vs Ethean solo restart](./peer-clients-fixed-genesis-vs-ethean-solo-2026-09-20.md)
- [Peer storage vs Ethean JSON snapshots](./peer-clients-storage-vs-ethean-json-2026-09-20.md)
- [Durable data-dir: genesis.json, SSZ, ethean.redb](./ethean-redb-ssz-data-dir-2026-09-20.md)
- [Data-dir process logs](./ethean-data-dir-run-logs-2026-09-20.md)
- [Console log colors](./ethean-console-log-colors-2026-09-20.md)
- [Console log verbosity](./ethean-log-verbosity-2026-09-20.md)
- [Startup ASCII banner](./ethean-startup-banner-2026-09-20.md)
- [--reset-chain empties data-dir](./reset-chain-wipes-data-dir-2026-09-20.md)
- [Dual mode: durable data-dir + ephemeral smoke](./dual-mode-persist-and-ephemeral-2026-09-20.md)
- [Recommended test baseline (Ream ops + ethlambda)](./recommend-test-baseline-ream-ethlambda-2026-09-20.md)
- [Long-run metrics + Grafana (pq-devnet-4)](./long-run-metrics-grafana-2026-09-20.md)
- [Richer Grafana monitors from live metrics](./ethean-grafana-richer-monitors-2026-09-20.md)
- [Grafana No data / 1970 date fixes](./grafana-no-data-and-1970-fix-2026-09-20.md)
- [Local finality solo long-run](./local-finality-solo-long-run-2026-09-20.md)
- [`--metrics` starts Grafana + Prometheus](./metrics-flag-starts-grafana-prometheus-2026-09-20.md)
- [Why :3000 / :9090 stay blank without Docker](./grafana-prometheus-need-docker-2026-09-20.md)
- [Docker engine HTTP 500 when Hyper-V is missing](./docker-engine-500-hyperv-not-installed-2026-09-20.md)
- [Local private PQ mesh (dial generated nodes)](./local-pq-mesh-private-dial-2026-09-20.md)
- [Fork digest override and local Status](./fork-digest-local-status-2026-09-19.md)
- [Status handshake outboxes (Status + blocks-by-root staging)](./status-handshake-outbox-2026-09-19.md)
- [QuicSwarm Status request_response wire](./quic-status-reqresp-wire-2026-09-19.md)
- [Blocks-by-root request_response wire](./blocks-by-root-reqresp-wire-2026-09-19.md)
- [D5 Type-2 gossip envelope gate](./d5-type2-gossip-envelope-gate-2026-09-19.md)
- [Blocks-by-root SignedBlock ingest](./blocks-by-root-signedblock-ingest-2026-09-19.md)
- [Blocks-by-root multi-hop parent catch-up](./blocks-by-root-multihop-catchup-2026-09-19.md)
- [Type-2 structural split and pool reseed](./type2-structural-split-pool-reseed-2026-09-19.md)
- [Type-1 recursive merge](./type1-recursive-merge-2026-09-19.md)
- [Sync-lag duty gate threshold](./sync-lag-threshold-2026-09-19.md)
- [Blocks-by-range request scaffold](./blocks-by-range-scaffold-2026-09-19.md)
- [pq-devnet-5 client run verification](./pq-devnet-5-client-run-2026-09-19.md)
- [ethean PATH command after cargo build](./ethean-path-command-after-build-2026-09-20.md)
- [UDP Status probe, data-dir, libclang](./udp-probe-data-dir-libclang-2026-09-19.md)
- [Release runbooks](./release/README.md)
- [Master branch consolidation](./master-branch-consolidation-2026-09-19.md)

Protocol evidence tree: [`../spec/README.md`](../spec/README.md).
