# Restore v0.1.53 release notes (changelog + binaries)

## Symptom

On `pamenarti/ethean` the `v0.1.53` GitHub Release body showed only the
**Binaries** table. The milestone text (Added / Changed / Fixed / Removed) that
operators saw at cut time was gone. The org fork briefly had a mixed copy; both
needed the same full body again.

## Cause

`.github/workflows/release-binaries.yml` merges a fresh `## Binaries` section
into the existing release body. The old merge logic treated “end of binaries”
as the next `## ` heading. Changelog sections use `### Added` / `### Changed`,
so a second publish (Windows overlay rebuild) dropped everything after the
first binaries block and left binaries-only notes.

## Fix

1. Re-wrote `v0.1.53` notes on **both** `pamenarti/ethean` and
   `ethean-labs/ethean`: Binaries table first, then the `CHANGELOG.md` `[0.1.53]`
   content.
2. Updated the merge step to strip only the prior `## Binaries` block (footer
   `See [docs/release/binaries.md]` or until the next `###` / non-Binaries
   `##`), then prepend the new binaries section so changelog prose survives
   re-runs.

## What 0.1.53 contains (summary)

- leanMultisig + `ethean-prover`, aggregator/proposer proofs, signed gossip
- leanMetrics v3 (`lean_*`) + Grafana interop dashboard
- Live `ForkChoiceStore`, FC tip / safe-target, vote ingest, durable rebuild
- `GET /lean/v1/chain/fork_choice` and `GET /lean/v1/events`
- Multi-platform release archives (incl. Windows leanVM overlays)

Links: https://github.com/pamenarti/ethean/releases/tag/v0.1.53 ·
https://github.com/ethean-labs/ethean/releases/tag/v0.1.53
