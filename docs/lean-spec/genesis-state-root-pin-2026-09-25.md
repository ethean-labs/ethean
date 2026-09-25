# leanSpec genesis state / block root pins (2026-09-25)

## Why

`EMPTY_BLOCK_BODY_ROOT` was already pinned. Full genesis **state root** (and the
sealed genesis **header / block root**) still needed an explicit match against
leanSpec fill output so a config-built node agrees with fixture / peer genesis.

## What landed

| Constant | Value source |
| --- | --- |
| `PROD4_GENESIS_STATE_ROOT` | `hash_tree_root(State)` for time=0, four prod-scheme validators |
| `PROD4_GENESIS_BLOCK_ROOT` | Sealed header root = leanSpec `test_first_post_genesis_…` slot-1 `parentRoot` (`0x7abe554d…`) |
| `PROD1_GENESIS_BLOCK_ROOT` | Sealed header root = leanSpec `test_genesis_single_validator` slot-1 `parentRoot` (`0xc95ab27e…`) |

Helpers: `prod_scheme_genesis(n)`, `seal_genesis_header` in
`crates/genesis/src/leanspec_pins.rs`.

Keys are the prod-scheme XMSS public keys from the pinned
`fixtures-prod-scheme` STF genesis cases (same bytes as leanSpec fill JSON).

## Evidence

```text
cargo test -p ethean-genesis --lib -- leanspec_pins
# sealed_header_root == 7abe554da998339edfd2ac2b86874b1eeb200f91f11a65701bfbbadc65e1f862
```

## Still open

- Hive sync suite re-run (Docker + public GHCR).
- Same-client 3-validator finality under prover load.
