# Phase 05 fixtures — state transition

Provenance for `ethean-transition` (leanSpec lstar state transition).

- Manifest: [manifest.toml](./manifest.toml)
- Lock: [`../../pins/phase-05.lock.toml`](../../pins/phase-05.lock.toml)
- Authority: leanSpec@0b7d33ec `state_transition.py`

## Coverage (this phase)

Unit tests in `ethean-transition`:

- `process_slots` advances N slots and caches zero `state_root`
- Non-future target → `BlockSlotNotInFuture`
- Distinct attestation data > `MAX_ATTESTATIONS_DATA` (8) → `AttestationDataLimit`
- `apply_block` with empty proof → `UnsupportedSignature`
- Structural `apply_block_unverified` updates header (state_root left zero)

## Not included

- Upstream binary fixtures / prod scheme tarball differentials
- XMSS-verified block vectors (Phase 07/08)
