# Protocol Authority and Compatibility Planning

## Purpose

This directory defines the planning gate for Ethean's Lean Consensus protocol migration. It does not select unresolved protocol values and does not authorize implementation. Its job is to make authority, compatibility, refresh, and decision ownership explicit before consensus-facing code is changed.

The evidence was inspected on 2026-09-19. The local research library is static source evidence; it is not a substitute for refreshing `leanSpec` main, its release assets, the active pq-devnet plan, or the selected cryptography repositories.

## Planning phases

The owner phases used by these documents are:

1. **P0 — Protocol Lock:** select and freeze one coherent upstream protocol profile.
2. **P1 — Types and Cryptography:** implement SSZ containers, roots, individual signatures, proof containers, and signer safety against the frozen profile.
3. **P2 — Transition and Fork Choice:** implement state transition, vote handling, fork choice, finality, and duties.
4. **P3 — Networking and Sync:** implement topics, codecs, message IDs, request/response, discovery, checkpoint bootstrap, and recovery.
5. **P4 — Interoperability and Release:** run pinned fixtures, negative mixed-version checks, seven-client interoperability, long-run testing, and release evidence.

An owner phase is accountable for resolving a decision before that phase starts. A decision may block later phases as well.

## Inspected peer snapshots

These commits are evidence snapshots, not protocol pins:

| Client | Branch | Inspected commit |
| --- | --- | --- |
| Ream | `master` | `b003b250f51c038cd5e16b8da02694ee0db1997e` |
| Zeam | `main` | `6495beb6b1a584e41c12b3569d9a509abd906259` |
| Qlean-mini | `master` | `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0` |
| ethlambda | `main` | `313b22d4aa15174b87319002d813926ab7eb6411` |
| Lantern | `main` | `440e34727ab3fb447de68d91a1e49be5327706e8` |
| gean | `main` | `b78f6d737f4df57a72d5e230635681235fda8024` |
| Peam | `master` | `6628e7a564098e592a49b9af0ad7b5dcda0a71fc` |

The snapshots are shallow. Several required submodules were not initialized. Static source findings do not establish successful builds or runtime interoperability.

## Evidence used

The planning set is based on:

- the local peer-library authority and protocol-surface notes;
- all seven peer audits;
- the cross-client comparison and open-work risk snapshot;
- the Ethean source-baseline audit;
- local pq-devnet-4, Poseidon cryptanalysis, and formal-verification notes.

The audited Ethean baseline was `f09b6edf6f305271c837127984489644468adda3`. It is a Beacon-era prototype and must not supply Lean protocol defaults.

## Document map

- [Authority Policy](./AUTHORITY_POLICY.md) defines source precedence, phase freezes, and fail-closed behavior.
- [Compatibility Ledger](./COMPATIBILITY_LEDGER.md) defines the machine-readable profile, exact pin requirements, fixture integrity, and compatibility fingerprint.
- [Upstream Refresh Policy](./UPSTREAM_REFRESH_POLICY.md) defines refresh triggers, protocol diffs, evidence retention, and phase-change handling.
- [Open Specification Decisions](./OPEN_SPEC_DECISIONS.md) enumerates every known blocking decision, its owner phase, evidence, resolution method, and impact.

## Current gate status

The protocol gate is **closed**. No coherent current profile has yet been proven across:

- a current `leanSpec` main commit and immutable fixture asset SHA-256;
- the selected XMSS and aggregation implementation revisions;
- exact public-key, signature, proof, field, hash, and lifetime parameters;
- the current fork-choice/finality generation;
- fork identity, topic grammar, message ID, Snappy, and request/response framing;
- discovery and checkpoint trust;
- durable signer-state guarantees;
- an exactly pinned Ethean compiler, dependency lockfile, and build image.

Observed peer values are deliberately retained as alternatives in the ledger and decision register. They are not defaults.

## Non-negotiable entry rule

P1 may start only when every P0 decision is resolved by authoritative evidence, the ledger has no unresolved consensus or wire field, every fixture archive has a verified SHA-256, and the computed compatibility fingerprint is recorded. A decoder, signer, verifier, node, or database opened with an unknown or mismatched fingerprint must refuse operation. There is no permissive fallback to Beacon behavior, a peer majority, a historical devnet, fake cryptography, or a branch name.
