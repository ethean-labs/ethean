# docs topic folders (2026-09-20)

Moved all dated development notes from `docs/*.md` into topic folders:

- `lean-spec/` — LeanSpec / fork choice / STF (28)
- `networking/` — Networking / QUIC / gossip / HTTP (26)
- `observability/` — Observability / metrics / Grafana (17)
- `pq-devnet/` — pq-devnet / operator run (13)
- `lean-crypto/` — leanSig / leanVM / aggregation (20)
- `storage/` — Storage / persist / prune (7)
- `hive-testing/` — Hive / fixture testing (6)
- `migration/` — Lean migration phases (25)
- `process/` — Process / README / git house rules (14)
- `peer-clients/` — Peer client references (6)
- `misc/` — Misc Lean research indexes (2)

Each folder has a `README.md` index. Root `docs/README.md` lists the folders.
Markdown links across the repo were rewritten to the new paths.
Evergreen manuals (`deployment.md`, `english.md`, …) stay at `docs/` root;
`docs/readme/` and `docs/release/` were already organized.
