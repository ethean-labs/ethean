# Release documentation

Operator-facing milestone notes live in the root
[`CHANGELOG.md`](../../CHANGELOG.md). Published cuts are GitHub Releases on
tags `vMAJOR.MINOR.PATCH` (first public milestones: `v0.1.12`, `v0.1.27`,
`v0.1.47`). Session write-ups under `docs/` stay detailed; the changelog is
the curated summary copied into each Release body.

- [gates.md](./gates.md) — promotion checklist and non-waivable gates
- [upgrade.md](./upgrade.md) — staged binary activation
- [rollback.md](./rollback.md) — fail-closed rollback and signer immutability
