# Peer reference clients

When a change is interop-critical (SSZ, signatures, aggregation, fork choice, gossip), look at Ream, Zeam, Qlean-mini, ethlambda, Lantern (`bitminetech/lantern`), gean, and Peam. Copy protocol behavior, not their code style or layout. leanSpec still wins if they disagree.

How Ream, ethlambda, and Zeam actually start and join a pq-devnet (shared genesis YAML, static ENRs, aggregator flag, D4 vs D5): [peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](peer-clients/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md).

## ReamLabs tooling (also reference)

Not clients. Use on every session that touches labs, fixtures, or operator drop-in:

| Tool | Repo | Role |
| --- | --- | --- |
| leanstart | https://github.com/ReamLabs/leanstart | Kind/Helm multi-client Lean mesh (genesis, keys, metrics, subnets) |
| lean-spec-tests | https://github.com/ReamLabs/lean-spec-tests | Shared leanSpec vectors; secondary to in-repo fixture pin |

Tracked summary: [reamlabs-leanstart-lean-spec-tests-2026-09-25.md](misc/reamlabs-leanstart-lean-spec-tests-2026-09-25.md). Local deep notes stay under `bazalinacaklar/reamlabs-tooling/` (gitignored).
