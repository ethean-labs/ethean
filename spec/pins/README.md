# Protocol pins

Phase lock files freeze the exact upstream inputs a migration phase may consume. They are evidence artifacts, not substitutes for `leanSpec` source.

## How locks work

1. Each phase that needs protocol identity keeps a `phase-NN.lock.toml` under this directory.
2. A lock lists **immutable** identifiers only: full git commits, tree hashes, release asset digests, byte lengths, and normative constant citations.
3. Later phases may copy a subset of Phase 00 fields into their own locks; they must not silently widen to a newer `main`.
4. Unresolved fields stay explicit. Absence of a value is a hard gate, not a license to invent defaults.

## Immutability rules

- Git pins use the full 40-hex commit ID (and tree hash when recorded).
- Annotated tags record both the tag object SHA and the underlying commit SHA.
- Fixture assets record: asset name, size in bytes, lowercase SHA-256, generator commit, and download URL for retrieval — **CI verifies the digest**, never the floating tag alone.
- A changed digest, size, or commit closes the gate until a reviewed lock revision lands.
- Do not commit multi-hundred-megabyte fixture archives into this repository; keep digests and manifests only.

## Authority order

Matches [`AUTHORITY_POLICY.md`](../../road-to/lean-consensus-migration/02-protocol/AUTHORITY_POLICY.md):

1. Frozen `leanSpec` commit + verified fixture release digests
2. Matching active pq-devnet plan / generated network artifacts (when selected)
3. Exact cryptography revisions named by that profile
4. Normative external protocol pins that `leanSpec` delegates
5. Pinned conformance / mixed-client results
6. Peer implementations at recorded evidence commits (observations only)
7. Ethean local policy (non-consensus, non-wire)
8. Historical Ethean code (inventory only)

Peer majority never resolves a contradiction against the frozen `leanSpec` tree or fixture digests.

## Files in this directory

| File | Purpose |
| --- | --- |
| `phase-00.lock.toml` | Research / compatibility snapshot lock |
| `phase-01.lock.toml` | Identity cleanup lock (inherits 00) |
| `phase-05.lock.toml` | Lean state transition lock (inherits 00–04) |
| `phase-06.lock.toml` | Lean fork choice lock (inherits 00–05; modified 3SF-mini / lstar) |
| `protocol-surface.toml` | Normative constants and wire/crypto surface for implementers |

## Refresh

Upstream `main` movement is tracked under the upstream refresh policy. A new freeze requires a new lock revision (or new phase lock), recomputed ledger fields, and reopened OSDs where semantic diffs appear.
