# examples

Lean Consensus client examples for the **Ethean Lean Consensus Client**.

## Status

The previous Beacon-era `integration_example.rs` was removed during migration planning. It depended on the legacy `panro` surface and is not a Lean interop example.

New examples are authored only after the matching replacement phase exits:

| Example class | Depends on | Owner phase |
| --- | --- | --- |
| Profile / genesis load | workspace + profile crates | 02–04 |
| SSZ encode / root check | types + fixtures | 03 |
| Local single-node tick | transition + fork choice | 05–06 |
| Validator duty dry-run | signer + duties | 07–09 |
| Two-node QUIC gossip | network crates | 10–11 |
| Metrics scrape smoke | `ethean-metrics` + exporter | 12 |

Until those phases land, this directory holds only this README so the required top-level `examples/` path remains documented.

## Rules

- Examples compile against public `crates/*` and `bin/ethean` APIs only.
- No imports from deleted legacy `src/` modules.
- No Beacon API, BLS, JSON consensus-root, or `panro` identifiers.
- Keep each example source file at most 300 lines.

See [deletion register](../road-to/lean-consensus-migration/05-retirement/DELETION_REGISTER.md) and [required directory policy](../road-to/lean-consensus-migration/05-retirement/REQUIRED_DIRECTORY_POLICY.md).
