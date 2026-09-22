# leanMultisig aggregation, verified block import (2026-09-23)

## Why

Blocks carry no individual signatures: one merged leanMultisig (Type-2)
proof binds every body attestation and the proposer (leanSpec
`lstar/signatures.py`). Ethean could not verify or produce that proof, so no
peer block could be imported and no local block was valid on a mesh. The
previous statement-based surface also did not match the spec: it bound a
body-root component, used the block slot for every attestation, kept the
proposer signature as an out-of-proof "sidecar", and accepted synthetic
`test-aggregate` proofs.

## What changed

- New crate `crates/multisig` (leanVM `e2592df4`, same pin as the other
  pq-devnet-4 clients): in-process verification, and proving in the new
  `ethean-prover` binary behind a supervised child-process protocol. See
  `crates/multisig/README.md` for the hardening (bounded LZ4, panic and
  stack isolation, binding checks).
- `ethean-crypto` keeps only the `AggregateVerifier` trait and parameters;
  the synthetic proofs, statement types, leanVM IPC stubs and the
  `ethean-leanvm-mock` binary are removed.
- `ethean-transition::apply_block` takes a verifier and rebuilds the spec
  component list from the parent registry: one component per body
  attestation (voters' attestation keys, `hash_tree_root(data)`, `data.slot`),
  then the proposer (proposal key, block root, block slot). Out-of-range
  voter or proposer indices are rejected before indexing.
- Node:
  - Attestation subnets carry `SignedAttestation`; each vote's XMSS signature
    is verified, and aggregators keep it in a bounded signature pool.
  - The aggregation topic carries `SignedAggregatedAttestation`; it is
    verified against the participants' keys before entering the pool.
  - `aggregation_duty` builds Type-1 proofs from fresh votes plus pooled
    proofs as children (greedy new coverage), re-verifies the result, and
    publishes it.
  - Proposals use only proved pool entries. The proposer signs the block root;
    the proof service wraps it as a singleton Type-1 and merges it after the
    attestation proofs. The block is verified with `apply_block` before it is
    imported locally and gossiped.
  - Proving runs on a background thread (`proof_service`), so the chain owner
    never blocks on a proof; results are collected at the start of each step.
  - Gossip block import is verified-only; unsigned `Block` payloads and the
    "advance head without state" shortcut are gone.
- `SignedAggregatedAttestation` SSZ now follows the spec field order
  (`data`, then the offset to `proof`); the old order was undecodable by peers.
- Release profile uses `panic = "unwind"` so verifier panics are contained.
- Linux only: PowerShell scripts and Windows-only notes removed; CI runs on
  Ubuntu; `fetch-leanspec-fixtures.sh` and `legacy-scan.sh` ported to bash.

## Evidence

- `crates/multisig/tests/end_to_end.rs`: native XMSS signatures (PROD keys)
  are accepted by leanSig, aggregate into Type-1, merge into Type-2 and
  verify; wrong message, slot, key set, component order and hostile bytes
  are rejected.
- `bin/ethean-prover/tests/process.rs`: proving through the child process,
  error reporting without losing the process, and respawn after a timeout.
- `crates/node/tests/aggregation_flow.rs`: three gossiped votes (one forged
  vote rejected) -> Type-1 on the aggregator -> verified on a peer -> block
  with a merged Type-2 proof -> tampered copy rejected, real block imported.
- Release timings on this machine: Type-1 verify ~26 ms, Type-2 verify
  ~33 ms, warm Type-1 prove ~0.33 s, Type-2 merge ~3.3 s, cold prover start
  ~4 s.

## Operating

- Build both binaries: `cargo build --release -p ethean -p ethean-prover`.
  The node finds `ethean-prover` next to its own executable, or via
  `ETHEAN_PROVER_BIN`.
- Aggregators and registry proposers need the prover; other nodes only
  verify. `ethean validator` prints what was found.

## Known gaps

- Attestation checks use the head state's registry and the structural parts
  of `validate_attestation`; the spec resolves keys from the target
  checkpoint's post-state through a fork-choice store Ethean does not run yet.
- A Type-2 merge takes seconds; with 4 s slots the proposer publishes late in
  the slot. This is leanVM's cost at this revision and affects every client.
