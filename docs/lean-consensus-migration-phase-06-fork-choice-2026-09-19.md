# Lean Consensus Migration — Phase 06 fork choice (2026-09-19)

## Summary

Added `ethean-fork-choice` (`crates/fork-choice`): a pure Lean fork-choice store for
**modified 3SF-mini / lstar** (Phase 00: not Goldfish). Node consensus stubs
`fork_choice.rs`, `finality.rs`, and `performance.rs` were deleted; the node
re-exports the new crate. Legacy `RealBLSAggregator` was renamed to `BlsAggregator`
so production paths no longer carry that symbol.

## Authority

| Item | Value |
| --- | --- |
| leanSpec | `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` |
| Citations | `fork_choice.py`, `timeline.py`, `containers/store.py` |
| Profile | `INTERVALS_PER_SLOT=5`, `GOSSIP_DISPARITY_INTERVALS=1`, 4s slots |
| Generation | `modified_3sf_lstar` (Goldfish = false) |

## Package surface

- `create_store(anchor_state, anchor_block, profile, opts)`
- `ForkChoiceStore::{on_tick, on_tick_with, on_block, on_attestation_data}`
- `head` / `safe_target` / `justified` / `finalized`
- Vote pools: `latest_new_attestations` / `latest_known_attestations` as
  `ValidatorIndex → AttestationData` (simplified vs leanSpec aggregate-proof maps)
- Head: weighted walk from justified using **known** votes; lex-larger root tie-break
- Safe target: same walk with 2/3 `min_score` on **pending** votes (interval 3)
- Tick: promote at interval 0 (with proposal) and 4; safe-target at 3
- Prune: drop finalized-away blocks; keep finalized ancestry

## Production safety

`ForkChoiceOpts::REQUIRE_PROOFS` (default) makes `on_attestation_data` return
`UnsupportedSignature` — no fake signature acceptance. Tests use `STRUCTURAL`.

## Node cleanup

- Deleted: `crates/node/src/consensus/{fork_choice,finality,performance}.rs`
- `consensus/mod.rs` re-exports `ethean_fork_choice`
- `bench` reduced to a stub; `RealBLSAggregator` → `BlsAggregator`

## Tests

```powershell
& $env:USERPROFILE\.cargo\bin\cargo.exe test -p ethean-fork-choice
```

5 unit tests passed (create/chain, tick promote, unknown parent, tie-break, proofs gate).

## Deferred

1. XMSS / leanMultisig proof verification on gossip attestations (Phase 07/08)
2. Full leanSpec `attestation_signatures` + `SingleMessageAggregate` pool model
3. Interval-2 aggregator bundling / broadcast (needs networking)
4. Upstream fork-choice fixture byte differentials
5. Wiring `ForkChoiceStore` into node client / sync coordinators (thin re-export only today)

## Artifacts

- `spec/pins/phase-06.lock.toml`
- `spec/fixtures/phase-06/{manifest.toml,README.md}`
- `docs/lean-consensus-migration-phase-06-fork-choice-2026-09-19.md` (this file)
- Local: `bazalinacaklar/phase-06-fork-choice.md`
