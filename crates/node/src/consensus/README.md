# consensus

Lean Consensus orchestration for the node.

- **State transition** — thin wrappers over `ethean-transition` (Phase 05).
- **Fork choice / finality** — stubs until Phase 06 store wiring.
- **Validator management** — XMSS registry helpers for API; no Beacon deposit/exit economics.
- **Slashing** — Beacon detectors removed; Lean equivocation not in Phase 05.
