# Full State SSZ encode/decode (2026-09-20)

## Problem

Block / attestation / checkpoint SSZ already round-tripped. `State::ssz_decode`
was deferred (“Phase 05”), so genesis SSZ load and peer-style state persistence
could not use the wire codec.

## Peer reference (field order)

Skimmed ethlambda `crates/common/types/src/state.rs` and Ream
`crates/common/consensus/lean/src/state.rs`. Same container order:

1. `config` (`genesis_time`)
2. `slot`
3. `latest_block_header`
4. `latest_justified` / `latest_finalized`
5. `historical_block_hashes` / `justified_slots`
6. `validators`
7. `justifications_roots` / `justifications_validators`

Peers use `libssz` / `ssz_derive`; Ethean keeps the in-tree `ethean-ssz` codec
(no peer crate copy).

## What landed

| Piece | Change |
| --- | --- |
| `ethean-ssz` | `decode_container_offsets`, `decode_hash32_list`, `InvalidFixedVector` |
| `ethean-types` `state/codec.rs` | Full `State::{ssz_encode,ssz_decode}` + roundtrip tests |
| `ethean-genesis` loader | Uses full decode; builder genesis SSZ roundtrip test |

## Operator note

`load_genesis_ssz(bytes, Some(root))` now works for non-empty Lean genesis
payloads produced by `GenesisBuilder` / future lean-quickstart imports.
