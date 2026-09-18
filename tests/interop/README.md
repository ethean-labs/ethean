# Interop and pinned-fixture testing

Future phases run **only** against fixtures whose digests match [`spec/fixtures/phase-00/manifest.toml`](../../spec/fixtures/phase-00/manifest.toml) (and later phase manifests when added).

## Preconditions

1. Phase lock present: [`spec/pins/phase-00.lock.toml`](../../spec/pins/phase-00.lock.toml).
2. Fixture archive downloaded to a **local cache** (not committed).
3. `sha256` and `size_bytes` match the manifest exactly.
4. Generator commit equals `leanSpec@0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` for Phase 00.

## Recommended flow

```text
1. Read manifest.toml → expected sha256, size_bytes, URL
2. Download asset to $ETHEAN_FIXTURE_CACHE (or CI cache)
3. Hash file; fail closed on mismatch
4. Extract to a versioned cache directory named by sha256 prefix
5. Discover cases; skip nothing that the selected profile marks mandatory
6. Record runner versions and output digests with the test report
```

## What not to do

- Do not use an unverified `latest` download as “good enough.”
- Do not regenerate fixtures from a different `leanSpec` commit and claim Phase 00 compatibility.
- Do not dual-decode multiple crypto generations in one profile.
- Do not treat peer client fixtures as authority over the pinned archive.

## Phase ownership

| Concern | Owner |
| --- | --- |
| Lock + manifest integrity | Phase 00 / P0 |
| SSZ / signature / proof vectors | Phases 03, 07, 08 |
| Transition / fork-choice vectors | Phases 05, 06 |
| Gossip / req-resp codec vectors | Phase 10 |
| Mixed-client matrix | Phase 12 / P4 |

Executable Rust interop harnesses land in later phases; this directory is the policy entry point until those crates exist.
