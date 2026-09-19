# Spec locks and fixtures

Tracked protocol evidence for the Lean Consensus migration. These paths are planning and CI inputs, not a runnable `leanSpec` checkout.

## Layout

| Path | Role |
| --- | --- |
| [pins/README.md](./pins/README.md) | How locks work, immutability rules, authority order |
| [pins/phase-00.lock.toml](./pins/phase-00.lock.toml) | Phase 00 frozen pins (resolved and unresolved sections) |
| [pins/phase-02.lock.toml](./pins/phase-02.lock.toml) | Phase 02 workspace / primitives / profile |
| [pins/phase-03.lock.toml](./pins/phase-03.lock.toml) | Phase 03 SSZ + Lean types |
| [pins/phase-04.lock.toml](./pins/phase-04.lock.toml) | Phase 04 genesis + 4s slot clock |
| [pins/protocol-surface.toml](./pins/protocol-surface.toml) | Normative constants and surfaces for later phases |
| [fixtures/README.md](./fixtures/README.md) | Fixture provenance policy |
| [fixtures/phase-00/](./fixtures/phase-00/) | Phase 00 fixture manifest (digests only; no tarball in git) |
| [fixtures/phase-03/](./fixtures/phase-03/) | Phase 03 SSZ/types fixture manifest |
| [fixtures/phase-04/](./fixtures/phase-04/) | Phase 04 genesis/clock fixture manifest |

## Related

- Authority policy: [`../road-to/lean-consensus-migration/02-protocol/AUTHORITY_POLICY.md`](../road-to/lean-consensus-migration/02-protocol/AUTHORITY_POLICY.md)
- Compatibility ledger: [`../road-to/lean-consensus-migration/02-protocol/COMPATIBILITY_LEDGER.md`](../road-to/lean-consensus-migration/02-protocol/COMPATIBILITY_LEDGER.md)
- Open decisions: [`../road-to/lean-consensus-migration/02-protocol/OPEN_SPEC_DECISIONS.md`](../road-to/lean-consensus-migration/02-protocol/OPEN_SPEC_DECISIONS.md)
- Interop test instructions: [`../tests/interop/README.md`](../tests/interop/README.md)

## Rule

Never treat a floating branch, `latest` tag alone, or peer majority as a pin. Digests and full 40-hex commits in the lock file win.
