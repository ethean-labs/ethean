# Lean peer-client research library, 2026-09-19

## Purpose

This research update prepares the evidence base for a later plan to convert Ethean from its Beacon-era prototype into an Ethereum Lean Consensus client. It does not select an implementation architecture or define a development sequence.

The detailed source notes are local under `bazalinacaklar/peer-client-library/` because peer clones and research extracts are intentionally excluded from Git. This tracked document records what was studied, how the evidence was qualified, and which findings must constrain future planning.

## Repositories inspected

The following shallow snapshots were cloned under `bazalinacaklar/peer-repos/` and audited:

- Ream, `b003b250f51c038cd5e16b8da02694ee0db1997e`
- Zeam, `6495beb6b1a584e41c12b3569d9a509abd906259`
- Qlean-mini, `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0`
- ethlambda, `313b22d4aa15174b87319002d813926ab7eb6411`
- Lantern, `440e34727ab3fb447de68d91a1e49be5327706e8`
- Gean, `b78f6d737f4df57a72d5e230635681235fda8024`
- Peam, `6628e7a564098e592a49b9af0ad7b5dcda0a71fc`

Each audit distinguishes executable implementation evidence from README claims, comments, historical design notes, and inference. The review covered manifests, lockfiles, source layout, state transition, fork choice, finality, SSZ, XMSS, recursive aggregation, validator duties, networking, persistence, checkpoint sync, APIs, metrics, fixtures, CI, Docker, security boundaries, performance assumptions, TODOs, and explicit deviations.

## Local research artifacts

The local library contains:

- one comprehensive audit for each of the seven clients;
- a cross-client comparison;
- a protocol-surface checklist for later planning;
- a time-bounded open issue and pull-request risk snapshot;
- an evidence-based audit of Ethean's current Beacon-era baseline;
- a README that records authority order, snapshot metadata, reading order, and refresh discipline.

The protocol checklist is intentionally not an implementation plan. It identifies every source and conformance decision that must be pinned before work begins.

## Common implementation surface

The inspected clients converge on these broad Lean client responsibilities:

- bounded SSZ consensus containers and SSZ hash-tree roots;
- a slot-oriented Lean state transition with historical roots and justification participation;
- vote-weighted fork choice, safe-target handling, and modified 3SF-style finality in the inspected generation;
- separate XMSS attestation and proposal keys;
- individual hash-based signatures, explicit participant sets, and recursive aggregate proofs;
- proposer, attester, and independent aggregator duties within a four-second slot;
- QUIC, Gossipsub, SSZ plus Snappy, and block, attestation-subnet, and aggregation topics;
- status exchange, blocks-by-root/range recovery, checkpoint or anchor sync, and persistent state;
- leanSpec-derived fixtures for SSZ, transitions, fork choice, signatures, and networking.

This convergence is implementation evidence, not protocol authority. Exact rules still come from the selected leanSpec release and matching cryptography and devnet pins.

## Critical differences and unresolved protocol questions

The snapshots do not implement one uniform protocol generation:

- Peam is materially older and retains devnet-2/devnet-3-shaped surfaces.
- Qlean-mini retains older proof-container and aggregation assumptions in important paths.
- Ream, Zeam, ethlambda, Lantern, and Gean expose different combinations of devnet-5 Type-1 and Type-2 proof behavior.
- Public-key and signature sizes differ between standalone leanSig and leanVM-internal XMSS generations.
- leanSig, leanMultisig, and leanVM commits are not uniformly pinned across clients.
- `MAX_ATTESTATION_DATA` appears as both 8 and 16 across notes and generations.
- Some clients use a hard-coded dummy fork identifier.
- Discovery, Gossipsub scoring, request/response framing details, and checkpoint trust are not fully consistent.
- Inspected clients still implement modified 3SF-mini behavior while roadmap material points toward PQ heartbeat and Goldfish.

Future planning must choose one exact leanSpec and fixture release, then match every cryptographic and networking dependency to it. Majority behavior among clients must not resolve a disagreement.

## Security and reliability findings

Several risks recur across implementations:

- XMSS is stateful, but durable duplicate-signing and rollback protection is incomplete or unclear in multiple clients.
- Checkpoint sync often verifies internal consistency without proving canonical-chain membership.
- Recursive proof production can consume most of a four-second slot and several gigabytes of memory.
- Slow, wedged, or non-cancellable native proving calls can block proposals or aggregation.
- Pending blocks, aggregate variants, unknown-parent graphs, and peer/request maps need explicit bounds and pruning.
- Test or Shadow builds may accept fake signatures or proofs; those paths must be structurally isolated from production.
- Mutable fixture downloads, branch-based cryptography dependencies, uninitialized submodules, and unpinned build images weaken reproducibility.
- Several clients expose unauthenticated operational or test-driver endpoints under permissive bind defaults.
- Long-running P2P behavior has produced measurable CPU and liveness failures, including Lantern stream-table scans and Zeam swarm wedges.

These findings should shape Ethean's threat model and acceptance tests before performance optimizations are selected.

## Ethean baseline

The current Ethean code compiles at the recorded baseline, but compilation is not Lean interoperability evidence. The source audit found that:

- package and binary identity still use `panro`;
- protocol types are hand-written Serde structures;
- consensus roots are SHA-256 hashes of JSON rather than SSZ hash-tree roots;
- state, validator lifecycle, balances, epochs, BLS, and finality retain Beacon assumptions;
- local WOTS and pseudo-Poseidon code do not match a pinned Lean cryptographic construction;
- aggregation is BLS-shaped rather than recursive XMSS proof aggregation;
- networking is largely mock or in-memory behavior without the Lean wire protocol;
- the production-named RocksDB backend acknowledges operations without persistence;
- APIs and configuration still advertise Beacon timing, committees, duties, and networks;
- no pinned leanSpec conformance runner or mixed-client interoperability harness exists.

Generic HTTP infrastructure, backend interfaces, bounded queue ideas, metrics plumbing, and selected storage abstractions may be reusable only after they are separated from protocol assumptions and verified.

## Evidence limits

The peer clones were shallow. Several repositories use uninitialized submodules, so their gitlink revisions were recorded but their nested source was not always available. Most findings are static source findings; they do not claim successful local builds or seven-client runtime interoperability.

Open GitHub work was captured through the public issue API on 2026-09-19. That snapshot is useful for identifying active failures and migrations but is neither complete nor normative.

The local roadmap notes also contain stale or contradictory devnet information. They remain useful warnings, not a substitute for refreshing the active leanSpec, pq-devnet plan, fixture asset hashes, and cryptography pins immediately before planning.

## Planning gate

No Lean implementation plan should be accepted until it records:

1. the exact target devnet or protocol generation;
2. the leanSpec commit and fixture asset hash;
3. matching leanSig, leanVM or leanMultisig revisions and parameters;
4. the exact SSZ and networking schemas;
5. resolution of the 8-versus-16 attestation-data limit;
6. the intended finality generation and fork-choice tests;
7. XMSS signer-state and duplicate-signing guarantees;
8. proof CPU, memory, cancellation, and deadline budgets;
9. checkpoint trust and restart-recovery invariants;
10. conformance, negative, recovery, long-running, and mixed-client interoperability tests.

The local `protocol-surface-checklist.md` is the detailed source for completing this gate.
