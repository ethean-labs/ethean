# Hive / leanSpec fixture consumer scaffold (2026-09-20)

## What landed

1. **`tools/release/fetch-leanspec-fixtures.ps1`**
   - Reads `spec/fixtures/phase-00/manifest.toml`
   - Copies `bazalinacaklar/fixtures-prod-scheme.tar.gz` when present, else downloads
   - Verifies `sha256` + `size_bytes`, extracts to `.cache/leanspec-fixtures/extracted`
2. **`ethean-spec-fixtures` crate**
   - Parses filled JSON envelopes (`FixtureFile` / steps / `rejectionReason`)
   - Maps leanSpec rejection tokens → `ForkChoiceError` (starting set)
   - Discovers `*.json` under `ETHEAN_LEANSPEC_FIXTURES`
3. **Committed sample** under `spec/fixtures/samples/fork_choice/` for CI without the 154 MiB archive
4. `.cache/` gitignored

## Operator recipe

```powershell
tools/release/fetch-leanspec-fixtures.ps1
$env:ETHEAN_LEANSPEC_FIXTURES = "$PWD\.cache\leanspec-fixtures\extracted"
cargo test -p ethean-spec-fixtures
```

## Hive matrix (still open)

ethereum/hive `simulators/lean` launches **client Docker images**
(`ream_devnet5`, …). Ethean is not registered there yet. Next matrix steps:

1. Dockerfile + Hive client YAML nametag for Ethean (devnet4/5)
2. Expose Lean HTTP endpoints Hive RPC suite expects
3. Full fork-choice / STF step runners against filled vectors (beyond rejection mapping)

Authority: leanSpec pin in `spec/fixtures/phase-00/manifest.toml`; peer
`ReamLabs/lean-spec-tests` is a secondary vector source.

## Honesty

- Sample JSON is structural only (no full `anchorState` / block execution yet)
- Fetch script does not vendor the tarball into git
- Fork-choice **rejection** steps against filled vectors: see
  [`leanspec-fc-rejection-runner-2026-09-20.md`](./leanspec-fc-rejection-runner-2026-09-20.md)
