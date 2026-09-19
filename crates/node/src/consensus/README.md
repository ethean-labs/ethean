# consensus

Lean Consensus orchestration for the node.

- **State transition** — thin wrappers over `ethean-transition` (Phase 05).
- **Fork choice / finality** — pure store from `ethean-fork-choice` (Phase 06,
  modified 3SF-mini / lstar; not Goldfish).
- **Validator management** — XMSS registry helpers for API; no Beacon deposit/exit economics.
- **Slashing** — Beacon detectors removed; Lean equivocation not yet wired.
