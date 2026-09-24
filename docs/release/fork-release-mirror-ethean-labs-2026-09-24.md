# Fork release mirror: pamenarti → ethean-labs (2026-09-24)

## Why Sync fork does not copy Releases

GitHub **Sync fork** (and a normal `git pull` of `master`) only moves **commits /
branches**. It does **not** copy:

- Git **tags** (unless you push them yourself)
- GitHub **Releases** (notes, Latest flag, binary assets)

Releases are API objects attached to a repository, not part of the git tree.
A private fork of `pamenarti/ethean` can show “up to date with
`pamenarti/ethean:master`” and still have **0 tags** and **No releases
published** until tags and Releases are created on `ethean-labs/ethean`.

## What we did

1. Added remote `ethean-labs` and pushed existing tags:
   `v0.1.12`, `v0.1.27`, `v0.1.47`, `v0.1.53`.
2. For each tag, recreated the Release on `ethean-labs/ethean` with the same
   title/body (download links rewritten to `ethean-labs`), marking `v0.1.53` as
   Latest.
3. Copied binary assets that exist on the source (`ethean-*` archives +
   `.sha256`). `v0.1.12` and `v0.1.27` had notes-only Releases upstream (no
   binaries); the mirror matches that.

Published: https://github.com/ethean-labs/ethean/releases

## Ongoing workflow

When cutting a new version on the personal repo (or wherever CI builds):

1. Tag and publish Release + binaries on the canonical build repo (today:
   `pamenarti/ethean` via `release-binaries` workflow).
2. Push the new tag to `ethean-labs`:
   `git push ethean-labs vX.Y.Z`
3. Mirror the Release (notes + `ethean-*` assets), or enable the same
   `release-binaries` workflow on `ethean-labs` so a tag push there builds and
   uploads directly.

Do not rely on the fork Sync button for Releases.
