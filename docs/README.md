# docs

Design notes, sprint write-ups, and working plans for the client. Root README should link anything here that a new contributor actually needs.

Lean R&D capture (public pointer only): [leanroadmap-local-notes.md](./leanroadmap-local-notes.md). The detailed extracts live in the local `bazalinacaklar/` folder.

Seven-client source research: [lean-peer-client-research-library-2026-09-19.md](./lean-peer-client-research-library-2026-09-19.md). The full per-client audits and planning checklist remain in the local research folder.

Code layout: [source-file-size-limit.md](./source-file-size-limit.md) (300-line cap, split by module).

Peer clients (reference, not a clone): [peer-reference-clients.md](./peer-reference-clients.md).

Language: [english.md](./english.md) (all in-repo text in English).

Git: [commit-after-each-file.md](./commit-after-each-file.md) (English commit after each file, even mid-prompt).

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
- [UDP Status probe, data-dir, libclang](./udp-probe-data-dir-libclang-2026-09-19.md)
- [Release runbooks](./release/README.md)
- [Master branch consolidation](./master-branch-consolidation-2026-09-19.md)

Protocol evidence tree: [`../spec/README.md`](../spec/README.md).
