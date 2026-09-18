# Protocol Authority Policy

## Scope

This policy governs every consensus, cryptography, wire, fixture, genesis, signer, and checkpoint decision in the Lean Consensus migration. It separates normative authority from implementation evidence and prevents a mixed-generation profile.

## Authority hierarchy

Use the first source that normatively defines the questioned surface:

1. **Frozen `leanSpec` commit and its verified fixture release.** The selected commit defines consensus containers, SSZ, state transition, fork choice, signature bindings, and Lean-owned networking fixtures.
2. **Matching active pq-devnet plan and generated network artifacts.** These define the deployed protocol generation, genesis/config schema, fork identity, role assignments, acceptance criteria, and interop packaging.
3. **Exact cryptography revisions selected by that profile.** `leanSig`, `leanVM`, or `leanMultisig` define key/signature serialization, XMSS instantiation, proof statements, recursion, security parameters, and lifecycle behavior.
4. **Normative external protocol pins.** The selected libp2p Gossipsub, discovery, QUIC, Snappy, and framing specifications govern details that `leanSpec` delegates explicitly.
5. **Pinned conformance and mixed-client results.** Reproducible fixture output and observed wire behavior can demonstrate that an interpretation matches the sources above.
6. **Peer implementations at recorded commits.** Ream, Zeam, Qlean-mini, ethlambda, Lantern, gean, and Peam are independent evidence for ambiguities, failure modes, and operational constraints.
7. **Ethean policy.** Ethean may choose local storage, scheduling, resource, API, and observability behavior only where higher authorities leave behavior non-consensus and non-wire.
8. **Historical Ethean code and documentation.** These are migration inventory only and have no authority over Lean behavior.

README text, comments, issue discussions, branch names, and a majority of peers do not override executable normative sources or fixtures.

## Following `leanSpec` main

Ethean follows `leanSpec` main for research and future-profile preparation, but never consumes a moving branch in a build:

- The upstream tracker records the exact fetched main commit.
- Protocol diffs are evaluated commit-to-commit.
- Fixture generation runs from the exact reviewed commit.
- A phase uses only an immutable commit and fixture asset digest entered in the compatibility ledger.
- “Latest,” an unqualified branch URL, or a mutable release asset is forbidden in CI and release builds.

Following main therefore means continuously evaluating main between freezes, not silently changing the active profile.

## Phase-freeze policy

### Freeze entry

Before P1, P2, P3, or P4 starts, the responsible phase must:

1. resolve every decision that can affect that phase;
2. update the machine-readable ledger;
3. verify source commits and fixture asset SHA-256 values;
4. regenerate and review the compatibility fingerprint;
5. run all prerequisite fixture suites with zero unknown mandatory types and zero unapproved skips;
6. record the upstream diff reviewed since the prior freeze.

### During a phase

The frozen compatibility fingerprint is immutable. Upstream main continues to be monitored, but new commits are queued for the next boundary. No dependency resolver, fixture downloader, submodule update, container build, or test job may substitute newer content.

### Security or interoperability exception

A newly disclosed consensus-safety, signature-safety, proof-soundness, or wire-split defect pauses the affected phase. The active freeze is not patched in place. The team creates a candidate profile, performs the full change-diff workflow, recomputes the fingerprint, reruns prerequisite tests, and explicitly supersedes the old freeze.

### Freeze exit

A phase exits only with:

- a ledger and fingerprint matching the tested binaries;
- fixture and negative-test evidence;
- all deviations classified as consensus, wire, operator policy, or local implementation;
- no unresolved decision marked as blocking the completed phase.

## Disagreement handling

When sources disagree:

1. Record each concrete value and its exact source revision.
2. Determine whether the disagreement is generation-specific, a local policy, a stale document, a fixture defect, or a true specification ambiguity.
3. Reproduce the relevant fixture or reference function from the candidate `leanSpec` commit.
4. Confirm the matching cryptography or network revision when the behavior is delegated.
5. Obtain an upstream specification decision when no authoritative executable answer exists.
6. Update the decision register and ledger only after evidence is reproducible.

Peer counting is prohibited. For example, five clients using eight attestation data values does not resolve a current `leanSpec` profile against a fixture that requires sixteen.

## Exactness requirements

Every selected pin must be immutable and complete:

- Git dependencies use full 40-hex commit IDs.
- Release assets use source URL, immutable release identity, byte length, and lowercase SHA-256.
- Container images use content digests, not tags.
- Rust, Zig, Go, C/C++, CMake, Python, `uv`, and code generators use exact versions where they participate in generated or built output.
- Submodules record both superproject gitlink and initialized nested commit.
- Lockfiles are part of the profile and contribute their SHA-256 to the compatibility fingerprint.
- Crypto records include construction, field, hash instance, dimensions, base, lifetime, activation, `log_inv_rate`, key size, signature size, proof bound, and serialization.
- Network records include fork identity derivation, exact topic bytes, message-ID preimage, compression mode, request/response framing, protocol IDs, response codes, and maximum decoded sizes.

An exact peer pin is evidence only. It becomes an Ethean profile pin only after the authority chain proves it matches the selected `leanSpec`.

## Fail-closed policy

Ethean must refuse to build, start, sign, verify, decode, connect, import, restore, or publish when any required compatibility field is unresolved or mismatched.

Mandatory refusals include:

- missing or incorrectly hashed fixtures;
- unsupported fixture type or unreported skipped case;
- unknown SSZ container, fork identity, proof generation, or crypto parameters;
- public-key, signature, proof, or participant shape mismatch;
- persisted data carrying another compatibility fingerprint;
- signer state that is absent, rolled back, concurrently leased, exhausted, or inconsistent with the active key;
- checkpoint data without the configured trust evidence;
- fake, Shadow, dummy, or test cryptography in a production profile;
- a moving dependency branch, mutable fixture URL, or unpinned generator;
- a network peer advertising an incompatible profile where silent coexistence could fork.

Failure must identify the mismatched field and expected fingerprint. It must not downgrade to Beacon constants, another peer's format, legacy devnet decoding, deterministic test keys, an ephemeral identity, or signature bypass.

## Local-policy boundary

Ethean may choose implementation details such as database engine, queue implementation, worker count, cache shape, metric backend, and proposer selection heuristics only when:

- the choice cannot alter accepted consensus objects or wire bytes;
- protocol maxima remain enforced;
- timing changes do not cross a normative duty boundary;
- the behavior is recorded as local policy rather than specification;
- mixed-client and adversarial tests show no compatibility regression.

Resource limits may be stricter only if they do not reject protocol-valid mandatory traffic needed for safety or liveness. Such limits require measured justification.
