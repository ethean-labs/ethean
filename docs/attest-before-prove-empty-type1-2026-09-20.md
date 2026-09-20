# Attest-before-prove and empty Type-1 pool proofs (2026-09-20)

## Problem

Local attester seeded the aggregate pool with the individual XMSS signature in
`PoolEntry.proof`. That blob could flow into merge / gossip as if it were a
Type-1 aggregate proof. Aggregator evaluate also ran **before** attest on the
same wall tick, so a fresh vote could not be proved until the next interval.

## What landed

| Piece | Change |
| --- | --- |
| `duty_attest` | Insert pool entry with **empty** `proof`; XMSS stays only on `AttestationSigned` |
| `duty_step` | Order: attest → aggregator ready → Type-1 prove → gossip queue → propose |
| `duty_aggregator` | Subnet = first set aggregation bit mod ACC (matches attester); hash fallback if no SSZ |

Gossip encode already refuses empty proofs (`aggregation_gossip`), so mesh publish
only happens after `try_prove_type1_for_root` succeeds (leanVM IPC or `test-aggregate`).

## Tests

```bash
cargo test -p ethean-node --lib duty_
```

## Follow-ups

- leanVM Split ABI (Type-2 → Type-1) skeleton
- Production leansig/leanVM in Hive image (external pins)
