# Attest head vs safe-target + fork_choice HTTP (2026-09-24)

## Why

Local attestations were voting `safe_target` for **both** head and target.
Lean interval-3 safe-target is the attestation **target**; the tip remains the
fork-choice **head**. Operators also had no Lean HTTP surface for store liveness
or pending/known vote counts.

## What landed

- `duty_attest`: `head = owner.head_root`, `target = safe_target` when set.
- `GET /lean/v1/chain/fork_choice` with live flag, roots, reorg_total, block
  and vote-pool sizes (published from the duty metrics refresh).
- Unit coverage in `ethean-rpc` for the route and JSON shape.

## Still external

- Fixture re-fill for empty-body dumps.
- Operator A2/A3 digests and bootnodes (paste only).

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/chain/fork_choice
```
