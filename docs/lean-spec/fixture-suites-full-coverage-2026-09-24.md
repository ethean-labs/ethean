# leanSpec fixture suites: full coverage — 2026-09-24

The pinned archive (`spec/fixtures/phase-00/manifest.toml`, leanSpec `0b7d33ec`)
holds twelve suites. `crates/spec-fixtures` used to run hand-picked fork-choice
and state-transition cases; it now enumerates every case of every suite Ethean
can execute, and each run reports counts and the names of unsupported vectors.

| Suite | Cases | Runner | Result |
| --- | --- | --- | --- |
| fork_choice | 123 (690 steps) | `driver.rs` (hive test_driver semantics) | all pass; `INVALID_SIGNATURE` steps need the node verifier (release node test) |
| state_transition | 74 | `driver.rs` | all pass |
| ssz | 59 | `suite_ssz.rs` | 51 pass (45 round trips, 6 rejections), 8 unsupported |
| networking_codec | 127 | `suite_networking.rs` | 95 pass, 32 unsupported |
| slot_clock | 25 | `suite_scalar.rs` | all pass |
| justifiability | 37 | `suite_scalar.rs` | all pass |
| poseidon_permutation | 8 | `suite_scalar.rs` | all pass |
| sync | 6 | `suite_scalar.rs` | all pass |
| verify_signatures | 3 | node `test_driver` tests (release) | all pass |
| verify_single_message_proofs | 2 | node `test_driver` tests (release) | all pass |
| api_endpoint | 16 | node `api_fixture_tests.rs` through the `/lean/v0` handlers | all pass |
| gossipsub_handler | 14 | not run | gossipsub mesh behaviour belongs to libp2p |

Unsupported ssz types: `AttestationSubnets` (Ethean has no ENR subnet bitvector),
`HashTreeLayer` / `HashTreeOpening` (XMSS internals kept opaque) and
`DecodeBitvector16` (no standalone bitvector decoder). Unsupported
networking vectors: `gossipsub_rpc` protobuf framing and `peer_id` derivation
(both owned by libp2p), so they are reported, not asserted.

## Divergences the ssz suite found and fixed

Every one of these changed bytes on the wire; none was visible to the JSON
fixtures the runners consumed before.

- `BlockBody` is a container with one variable field, so its encoding starts
  with the offset `4`; Ethean emitted the bare attestation list. Every block
  Ethean produced was 4 bytes short and undecodable by a spec client.
- `MultiMessageAggregate` is likewise a container around the proof bytes;
  Ethean emitted the raw bytes, so `SignedBlock` also diverged.
- `State.validators` is a list of fixed-size elements and is encoded as a plain
  concatenation; Ethean wrote per-element offsets, so served finalized states
  (`/lean/v0/states/finalized`) did not decode in hive.
- `Validator.index` has no SSZ-level bound; Ethean rejected large indices at
  decode time.
- `SignedAttestation` and `Signature` roots hash the signature as the leanSpec
  `Signature { path, rho, hashes }` container (`crates/types/src/xmss_root.rs`,
  node-list limit `2^17`), not as a byte vector.

Because persisted blocks and states change shape, the durable schema id moved
from `ethean-lc-d5-v1` to `ethean-lc-d5-v2`; existing `--data-dir` trees must
be recreated.

The API suite also requires three gauges from the leanSpec node registry that
the pinned leanMetrics table does not list
(`lean_attestation_aggregate_coverage_{validators,subnets,diff_validators}`);
they are registered as an extra table (`crates/metrics/src/lean/spec_extra.rs`)
and recorded per imported or proposed block.

Also confirmed: Ethean's genesis built from the fixture key set is byte-equal
to the state leanSpec serves for a genesis checkpoint (`sync` suite), which
closes the genesis-root pin left open in the genesis note.
