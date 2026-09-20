# Peer reference clients

When a change is interop-critical (SSZ, signatures, aggregation, fork choice, gossip), look at Ream, Zeam, Qlean-mini, ethlambda, Lantern (`bitminetech/lantern`), gean, and Peam. Copy protocol behavior, not their code style or layout. leanSpec still wins if they disagree.

How Ream, ethlambda, and Zeam actually start and join a pq-devnet (shared genesis YAML, static ENRs, aggregator flag, D4 vs D5): [peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md](peer-clients/peer-clients-ream-ethlambda-zeam-devnets-2026-09-20.md).
