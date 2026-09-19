# ethean-fork-choice

Pure Lean fork-choice store for the **lstar / modified 3SF-mini** generation
(Phase 00: not Goldfish). Authority: leanSpec `0b7d33ec` `fork_choice.py` /
`timeline.py` / `containers/store.py`.

## Surface

- `create_store` — anchor block + post-state
- `on_tick` — interval advance, vote promotion, safe-target update
- `on_block` — import block with caller-supplied post-state (no STF re-run)
- `on_attestation_data` — structural vote ingest (proofs deferred / opts)
- Head walk from justified root using **known** votes; lex root tie-break
- Prune blocks finalized away while keeping finalized ancestry

## Non-goals

No networking, no XMSS verify, no Goldfish, no Beacon Casper epoch finality,
no dual engines.
