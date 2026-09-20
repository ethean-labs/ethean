# Root README overview / architecture rewrite (2026-09-20)

Expanded the early root [README.md](../../README.md) so newcomers see **what Ethean
is trying to do**, not generic Beacon marketing copy.

## Changes

- **Overview** — Lean goals (leanSig, aggregation, 3SF, 4s slots, 1 ETH stake
  direction), what the binary does today (D4 default, dual mode, QUIC, metrics,
  leanSpec fixtures), fail-closed crypto gates.
- **Key features** — project-specific bullets (ChainOwner, dual mode, pq-devnet
  labels, `/lean/v1`, house rules). Removed old “Eth2 PoS / WebSocket” fluff.
- **Architecture** — layer table matching the real crate split; links to
  `docs/readme/architecture.md` and the migration library.
- Companion [readme/architecture.md](./readme/architecture.md) gained a short
  **Intent** section.

License and below were not modified.
