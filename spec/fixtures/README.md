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
| [phase-02/](./phase-02/) | Workspace / primitives / profile |
| [phase-03/](./phase-03/) | Canonical SSZ + Lean types |
| [phase-04/](./phase-04/) | Genesis + 4s slot clock |
| [phase-05/](./phase-05/) | Lean state transition |
| [phase-06/](./phase-06/) | Lean fork choice (modified 3SF-mini / lstar) |
| [phase-07/](./phase-07/) | XMSS signer |
| [phase-08/](./phase-08/) | leanVM aggregation |
| [phase-09/](./phase-09/) | Validator duties |
| [phase-10/](./phase-10/) | QUIC gossip |
| [phase-11/](./phase-11/) | Storage / sync |
| [phase-12/](./phase-12/) | API / observability |
| [phase-13/](./phase-13/) | Security / performance / release |
| [samples/](./samples/) | Tiny committed JSON envelopes for CI (`ethean-spec-fixtures`) |

Fetch full prod-scheme archive (gitignored cache):
[`../../tools/release/fetch-leanspec-fixtures.ps1`](../../tools/release/fetch-leanspec-fixtures.ps1).

See also [`../pins/phase-00.lock.toml`](../pins/phase-00.lock.toml), [`../pins/phase-03.lock.toml`](../pins/phase-03.lock.toml), [`../pins/phase-05.lock.toml`](../pins/phase-05.lock.toml), [`../pins/phase-06.lock.toml`](../pins/phase-06.lock.toml), and [`../../tests/interop/README.md`](../../tests/interop/README.md).
