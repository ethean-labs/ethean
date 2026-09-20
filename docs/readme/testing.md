# Testing

How to run the workspace test suites.

Short overview: [root README — Testing](../../README.md#testing).

## Running tests

```bash
# Entire workspace
cargo test

# One crate
cargo test -p ethean-fork-choice
cargo test -p ethean-spec-fixtures

# Show stdout
cargo test -- --nocapture
```

## What we cover

- Unit tests inside Lean crates (types, SSZ, fork choice, transition, …)
- Integration / interop suites under `tests/`
- LeanSpec fixture runners in `crates/spec-fixtures` (locked FC / STF vectors)

Session notes for locked vectors live as `docs/leanspec-*.md`. Cryptography tests
exercise leanSig / XMSS surfaces and stay fail-closed until production backends
link.

## Related

- [development.md](./development.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
