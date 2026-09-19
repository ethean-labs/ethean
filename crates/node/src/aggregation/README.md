# aggregation

Node-side Type-1 / Type-2 pool, deterministic selection, budgets, and prover worker facade.

- Proofs are inserted only after local verify (`ethean-crypto`).
- Proving must not run on the chain-owner tick; this module is a sync facade until a process sandbox lands.
- Production leanVM is fail-closed; default `test-aggregate` binds statements for unit tests only.
