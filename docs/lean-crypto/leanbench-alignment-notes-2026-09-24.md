# leanBench alignment notes (2026-09-24)

## Status

leanBench API modes and cross-machine benches are still published externally.
Ethean does **not** invent digests, bootnodes, or bench targets.

## In-repo pins that already match the leanBench default

| Item | Value | Where |
| --- | --- | --- |
| leanVM rev | `e2592df4…` | `ethean-crypto` / `ethean-multisig` |
| `LOG_INV_RATE` | `2` | `crates/crypto/src/aggregation/bindings.rs` |
| FC fixture path | `ForkChoiceOpts::STRUCTURAL` | node live store + spec-fixtures |
| Production attest proofs | `ForkChoiceOpts::REQUIRE_PROOFS` | fail-closed when proofs required |

## When leanBench goes live

1. Diff generation / `LOG_INV_RATE` / Type-1–2 component caps against the bench
   page; bump pins only with a peer-shared rev.
2. Document any API mode flags under `docs/lean-crypto/` (no invented numbers).
3. Keep operator A2/A3 paste slots empty until published.

Related: [pq-devnet-operator-plug-in-checklist-2026-09-20.md](../pq-devnet/pq-devnet-operator-plug-in-checklist-2026-09-20.md).
