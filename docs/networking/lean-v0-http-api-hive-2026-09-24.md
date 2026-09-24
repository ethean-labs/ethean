# Lean `/lean/v0` HTTP API (hive interop, 2026-09-24)

## What landed

| Piece | Change |
| --- | --- |
| `ethean-rpc` HTTP | Request-line + headers, `Content-Length` bodies (up to 64 MiB), `Accept`, keep-alive / close, exact `Content-Type` |
| Hive routes | `GET /lean/v0/{health,checkpoints/justified,fork_choice,states/finalized,blocks/finalized}` and `GET/POST /lean/v0/admin/aggregator` |
| Aliases | `/lean/v1/*` of the same paths; `/eth/` still 404 |
| Snapshot | `ForkChoiceView` on `ApiSnapshot`, published from `ChainOwner` on each metrics refresh |
| Genesis SSZ | Blank-proof `SignedBlock` + canonical state (`latest_block_header.state_root` zeroed, matching ethlambda) |

JSON numbers for slots; roots are `0x` + 64 lowercase hex. SSZ routes return `application/octet-stream` with no charset.

## Fresh-node contract (hive `rpc_compat`)

- `fork_choice.nodes` is exactly one genesis node: slot 0, parent `0x00…00`, proposer 0, weight 0.
- `head == justified.root == finalized.root == safe_target ==` that node root.
- `validator_count > 0` (never JSON null).
- `states/finalized` + `blocks/finalized` 200 with `block.slot == state.slot`, `block.state_root == hash_tree_root(canonical_state)`, `hash_tree_root(block) == fork_choice.finalized.root`.

503 until the chain has a sealed genesis / head. 404 if a finalized SSZ blob is missing.

## Weights

`ForkChoiceStore` is still optional on `ChainOwner`. When the store is present, node weights come from `block_weights_from_known`. Otherwise nodes are listed from durable / genesis blocks with **weight 0** (not faked).

Aggregator POST flips an atomic on `SharedApiState`; the duty-loop metrics publish copies it onto `ChainOwner.is_aggregator`.

## Smoke

```bash
ethean start --ticks 3 --until-signal --http-address 127.0.0.1 --http-port 5052 --ephemeral
curl -s http://127.0.0.1:5052/lean/v0/health
curl -s http://127.0.0.1:5052/lean/v0/checkpoints/justified
curl -s http://127.0.0.1:5052/lean/v0/fork_choice
curl -s -D - http://127.0.0.1:5052/lean/v0/states/finalized -o /tmp/state.ssz
curl -s -D - http://127.0.0.1:5052/lean/v0/blocks/finalized -o /tmp/block.ssz
curl -s http://127.0.0.1:5052/lean/v0/admin/aggregator
```

Metrics stay on `:9100/metrics`.
