# Spec fixtures

This tree holds **provenance manifests** for upstream fixture archives used by conformance and interop work.

## Rules

1. Do **not** commit large fixture tarballs (for example the ~154 MiB production scheme archive).
2. Every phase directory must contain a `manifest.toml` with immutable digests, sizes, generator commit, and retrieval URL.
3. CI downloads the asset, verifies `sha256` and `size_bytes`, then extracts to a local cache outside git.
4. A floating release tag (`latest`) may appear in the URL for convenience; the digest is the pin.
5. Local modifications to extracted fixtures are forbidden for profile claims.

## Layout

| Path | Role |
| --- | --- |
| [phase-00/](./phase-00/) | Phase 00 production-scheme fixture lock |

See also [`../pins/phase-00.lock.toml`](../pins/phase-00.lock.toml) and [`../../tests/interop/README.md`](../../tests/interop/README.md).
