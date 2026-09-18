# Open Specification Decisions

## Purpose

This register lists protocol decisions that **block migration phases** until they are closed with authoritative evidence. Peer implementations supply observations only; they do not select values.

Primary evidence inputs:

- [COMPATIBILITY_LEDGER.md](./COMPATIBILITY_LEDGER.md) — machine-readable profile fields, observed alternatives, and peer snapshot commits.
- Local research checklist: `bazalinacaklar/peer-client-library/protocol-surface-checklist.md` — section-by-section protocol surface inventory with peer file paths and stability labels.

Closure requires a frozen `leanSpec` commit, an immutable fixture release with independently verified SHA-256, regenerated vectors, ledger updates, and owner-phase approval. Until then every row below remains **`status: unresolved`**.

## Decision summary

| ID | Decision | Observed alternatives | Authority to resolve | Blocked phases | Status |
| --- | --- | --- | --- | --- | --- |
| OSD-001 | `MAX_ATTESTATION_DATA` | **8** vs **16** unique attestation-data values per block | Selected `leanSpec` pin + block-limit fixtures + active pq-devnet config | P03 (types), P05 (transition), P08 (aggregation), P09 (duties), P10 (gossip budgets) | unresolved |
| OSD-002 | XMSS public-key width | **32** bytes (leanVM internal) vs **52** bytes (standalone leanSig) | Selected crypto revision pin + SSZ/key fixtures | P03, P07 (signer), P04 (genesis keys) | unresolved |
| OSD-003 | XMSS signature width | **1,208** bytes vs **2,536** bytes | Same as OSD-002 + verify-signatures fixtures | P03, P07, P08, P10 | unresolved |
| OSD-004 | Crypto stack | **standalone leanSig** vs **leanVM internalized XMSS** | `leanSpec` block/XMSS containers + chosen `leanSig`/`leanVM` commits | P03, P07, P08, P11 (storage key schema) | unresolved |
| OSD-005 | Block proof envelope | **Type-1** (per-message proofs + proposer sig) vs **Type-2** (one multi-message block proof) | `leanSpec` block/proof containers + signature fixtures | P03, P05, P08, P09, P10 | unresolved |
| OSD-006 | Fork choice / finality | **modified 3SF-mini** vs **PQ heartbeat / Goldfish** | Selected `leanSpec` fork-choice branch + tick/finality fixtures | P05, P06 (fork choice), P09, P11 | unresolved |
| OSD-007 | Snappy profile | **raw Snappy** (gossip) vs **framed Snappy** (req/resp) | `leanSpec` networking codec fixtures | P10, P11 | unresolved |
| OSD-008 | Fixture release identity | No verified archive SHA-256 for the active profile | Immutable `leanSpec` release asset + generation command + manifest digest | P00, P03–P11, P4 interop | unresolved |
| OSD-009 | Observability contract | **leanMetrics** revision TBD | Pinned `leanMetrics` for selected pq-devnet + Hive/interop log contract | P06 (observability), P09, release acceptance | unresolved |

Owner phases follow [02-protocol/README.md](./README.md): **P0** Protocol Lock, **P1** Types/Crypto, **P2** Transition/Fork Choice, **P3** Networking/Sync, **P4** Interop/Release. Phase numbers in the table use migration filenames (`phases/03-…` etc.).

Peer evidence commits (baseline snapshots, not pins) are recorded in `COMPATIBILITY_LEDGER.md` under `peer_evidence` and `observed_*` fields.

---

## OSD-001 — `MAX_ATTESTATION_DATA`: 8 vs 16

**Status:** unresolved  
**Owner phase:** P0 — Protocol Lock  
**Blocked phases:** [03-canonical-ssz-and-types](../phases/03-canonical-ssz-and-types.md), [05-state-transition](../phases/05-state-transition.md), [08-leanvm-aggregation](../phases/08-leanvm-aggregation.md), [09-validator-and-node-duties](../phases/09-validator-and-node-duties.md), [10-quic-gossip-and-reqresp](../phases/10-quic-gossip-and-reqresp.md)

### Observed alternatives

| Value | Peer evidence (inspected commit) | Source path / note |
| --- | --- | --- |
| **8** | Ream `b003b250f51c038cd5e16b8da02694ee0db1997e` | devnet-5-shaped block attestation limits |
| **8** | Zeam `6495beb6b1a584e41c12b3569d9a509abd906259` | block builder / type bounds |
| **8** | ethlambda `313b22d4aa15174b87319002d813926ab7eb6411` | `crates/common/types` constants |
| **8** | Lantern `440e34727ab3fb447de68d91a1e49be5327706e8` | `consensus/containers.h`, signature validation |
| **8** | gean `b78f6d737f4df57a72d5e230635681235fda8024` | block builder selection |
| **16** | Qlean-mini `55b6eb3c14dfee3aa1b1bb855a55be4723b7bdd0` | `chain_spec.hpp`, attestation-limit fixtures |
| **16** (objectives) / **8** (summary) | pq-devnet-4 plan | local `pq-devnet-4.md` contradiction — checklist §4 block-body limits |

### Authority to resolve

1. Freeze one `leanSpec` main commit (see ledger `authority.lean_spec.selected_commit`).
2. Run block attestation limit fixtures: zero, typical, duplicate-data, overflow, and coalescing-negative cases.
3. Confirm SSZ list bound and semantic rejection reason match generated output.
4. Align with the active pq-devnet plan generation named by that pin.

Peer majority does not resolve the contradiction. Five clients at eight does not override a fixture requiring sixteen.

### Checklist and ledger references

- Checklist §4 “Block body limits”, §14 “Devnet-5 vectors”.
- Ledger `consensus.max_attestation_data`, `consensus.observed_max_attestation_data`.

### Blocking impact

Until closed: block body SSZ bounds, transition validation, proposer attestation gathering, aggregate pool sizing, gossip decode budgets, and proof workload estimates are undefined.

---

## OSD-002 — Public-key width: 32 vs 52 bytes

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P03, P04, P07, P11

### Observed alternatives

| Width | Generation | Peer evidence | Crypto dependency commit (observed) |
| --- | --- | --- | --- |
| **52** bytes | standalone leanSig Dim46 | Ream, Zeam, ethlambda, Lantern | leanSig `15cbdd43ec8525aa43fea2f42cafc5ed366084ae` |
| **32** bytes | leanVM internal V=42 | gean | leanVM `a5909d18647de6aed38640c098d9177fab2bf36a` |
| **52** bytes (historical) | devnet-2 leanSig Dim64 | Peam `6628e7a564098e592a49b9af0ad7b5dcda0a71fc` | leanSig `73bedc26ed961b110df7ac2e234dc11361a4bf25` — incompatible, not a candidate default |

gean explicitly documents that V=42 32-byte keys are **incompatible** with Dim46 leanSig keys.

### Authority to resolve

Selected `leanSpec` Validator container + genesis key artifacts + public-key serialization fixtures from the pinned crypto revision (`leanSig` or `leanVM` as named by the profile).

Ledger fields: `crypto.public_key_bytes`, `crypto.observed_generations`.

Checklist §6 “Crypto instantiation”, §6 “Key generation and serialization”.

### Blocking impact

Validator SSZ layout, genesis YAML, status/checkpoint registry comparison, proof public inputs, and persisted key schema cannot be implemented.

---

## OSD-003 — Signature width: 1,208 vs 2,536 bytes

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P03, P07, P08, P10

### Observed alternatives

| Width | Hash / field | Peer evidence | Observed commit |
| --- | --- | --- | --- |
| **2,536** bytes | Poseidon1, Dim46 leanSig | Ream, ethlambda, Lantern, Zeam | leanSig `15cbdd43…`, leanVM Type-2 glue `e2592df4…` (Zeam/ethlambda) |
| **1,208** bytes | Poseidon2, KoalaBear V=42 | gean | leanVM `a5909d1…` |
| **3,112** bytes | historical Peam devnet-2 | Peam only | leanSig `73bedc26…` — evidence of another generation |

Resolution is coupled to OSD-002 and OSD-004. Malformed-length vectors must reject before expensive verification.

Ledger: `crypto.signature_bytes`, `crypto.observed_generations`.

Checklist §6 signing payloads, §7 proof encoding, §14 signature/proof conformance.

---

## OSD-004 — Standalone leanSig vs leanVM internalized XMSS

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P03, P07, P08, P11, toolchain/dependency pinning (P02)

### Observed alternatives

| Stack | Construction | Public key | Signature | Peers | Observed pins |
| --- | --- | --- | --- | --- | --- |
| **standalone leanSig** | generalized XMSS aborting target-sum, Poseidon1, Dim46 | 52 B | 2,536 B | Ream, Zeam, ethlambda, Lantern | leanSig `15cbdd43…`; multisig/Type-2 via leanVM `e2592df4…` on several peers |
| **leanVM internalized XMSS** | leanVM internal XMSS, Poseidon2, KoalaBear V=42 | 32 B | 1,208 B | gean | leanVM `a5909d1…` |
| **historical leanSig + leanMultisig** | Dim64 devnet-2 | 52 B | 3,112 B | Peam | leanSig `73bedc26…`, leanMultisig `e4474138…` |

This is not a packaging preference. The stacks differ in field, hash instance, serialization, proof statements, and cross-verification behavior.

### Authority to resolve

1. Active pq-devnet plan names the production crypto generation.
2. `leanSpec` block/XMSS containers and verify-signatures fixtures must round-trip with the selected repository revision.
3. Generated genesis keys and interop vectors must match without dual decoders.

Ledger: `crypto.construction`, `crypto.signature_repository`, `crypto.aggregation_repository`, full `observed_generations` object.

Checklist §1 “Generation-specific block envelope”, §6 entire section, §7 leanMultisig/leanVM, §15 pinned build graph.

### Blocking impact

Dependency selection, signer glue, proof APIs, block production, storage of key material, and mixed-client matrix scope.

---

## OSD-005 — Type-1 vs Type-2 block proofs

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P03, P05, P08, P09, P10

### Observed alternatives

| Envelope | Description | Peer evidence |
| --- | --- | --- |
| **Type-1** | Per-attestation aggregate proof list + separate proposer signature | Qlean-mini `55b6eb3c…` — `block_signatures.hpp`, per-proof list |
| **Type-2** | Single multi-message block proof covering attestation components and proposer | Ream, Zeam, ethlambda, Lantern, gean — block builder / verify paths |
| **devnet-2 single-message** | Older Peam block shape | Peam — not current target |

Observed `leanSpec` gitlinks also split: Zeam/Lantern `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54`; gean fixture source `eca701efeb5931010fe63925cd203c9ee55b2dbc`.

### Authority to resolve

Frozen `leanSpec` block and XMSS container declarations + SSZ/signature fixtures defining component order, empty-proof rules, and proposer inclusion.

Ledger: `crypto.type1_container`, `crypto.type2_container`, `crypto.block_component_order`, `crypto.proof_max_bytes`.

Checklist §1 “Generation-specific block envelope”, §7 “Multi-message/block-level proofs” `[D5-UNSTABLE]`, §14 devnet-5 vectors.

### Blocking impact

Block SSZ layout, verification order, gossip payload size, storage schema, proposer duty implementation, and interoperability claims.

---

## OSD-006 — Modified 3SF-mini vs PQ heartbeat / Goldfish

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P05, P06 (when authored), P09, P11, long-run acceptance

### Observed alternatives

| Generation | Behavior | Peer evidence |
| --- | --- | --- |
| **modified 3SF-mini + latest-vote GHOST** | Tick-driven vote promotion, safe target, lexicographic tie-break, adjacency finalization | All audited devnet-5-shaped clients at baseline commits |
| **PQ heartbeat / Goldfish direction** | Roadmap and ethlambda prototype notes; not uniformly implemented | ethlambda labels Goldfish/RLMD-GHOST/BFT as later work; leanroadmap research tracks |

Local checklist marks this `[D5-UNSTABLE]`. Peers agree on responsibilities (latest vote, target selection) but document local deviations.

### Authority to resolve

Selected `leanSpec` fork-choice branch/commit named by the current pq-devnet plan. Full fixture families: ticks, vote promotion, safe target, reorg, equivocation, tie-break, justification, finalization.

Ledger: `consensus.fork_choice`, `consensus.finality`, `consensus.observed_finality`.

Checklist §5 fork choice/finality, `[BEACON-REJECT]` Casper/epoch imports.

Beacon Casper FFG, epoch checkpoints, and 32-slot epoch mapping are forbidden fallbacks.

### Blocking impact

All state transition visibility rules, duty timing, storage pruning floors, checkpoint semantics, and conformance suite selection.

---

## OSD-007 — Snappy: raw gossip vs framed request/response

**Status:** unresolved  
**Owner phase:** P0 (profile); P3 implementation  
**Blocked phases:** P10, P11

### Observed alternatives

| Surface | Encoding | Peer evidence |
| --- | --- | --- |
| **Gossip: raw Snappy** over SSZ | No varint length prefix | Qlean-mini, Lantern, gean, ethlambda, Zeam, Peam — gossip codec modules |
| **Req/resp: framed Snappy** | Varint length + Snappy chunks | Same peers — req/resp codec paths |

The disagreement is not whether gossip and req/resp differ; peers broadly agree they do. Remaining gaps are **framing edge cases**: empty status bodies, multi-root responses, partial ranges, error chunks, stream termination, trailing bytes, and maximum decoded sizes (ledger observes 10 MiB vs ~12 MiB caps).

### Authority to resolve

`leanSpec` networking codec fixtures for gossip and req/resp independently. Byte-identical reproduction with at least two current peers for each vector family.

Ledger: `network.gossip_compression`, `network.request_compression`, `network.request_framing`, `network.observed_compression`, `network.observed_max_decoded_bytes`.

Checklist §2 “SSZ plus Snappy framing”, §10 response safety, §14 networking codec conformance.

Related unresolved wire decisions (not separate rows above but coupled): gossip **message-ID preimage order** (Lantern/gean vs Qlean-mini — checklist §9) and **fork identifier bytes** (ledger `network.observed_fork_identifiers`).

### Blocking impact

Codec implementation, decode bomb budgets, sync streams, mixed-client gossip deduplication, and negative interop tests.

---

## OSD-008 — Fixture release SHA-256 unresolved

**Status:** unresolved  
**Owner phase:** P0  
**Blocked phases:** P00 research lock, P03–P11, P4 interop/release

### Observed state

The compatibility ledger records:

```yaml
fixtures:
  status: unresolved
  release_identity: null
  asset_url: null
  asset_sha256: null
  generator_commit: null
```

Peer fixture provenance is inconsistent:

| Peer | Fixture behavior | Observed note |
| --- | --- | --- |
| gean | pins generator source | `leanSpec` `eca701efeb5931010fe63925cd203c9ee55b2dbc` |
| Zeam, Lantern | gitlink | `8b4ebbea7bb011b80aadfe4e59da2a66c1a86d54` |
| ethlambda | downloads mutable release | `latest` URL risk — checklist §14 fixture provenance |
| Qlean-mini | cites spec commit for one bound | aggregate bound citation `6430aaf7…`; generator not fully pinned |

No single archive SHA-256 has been verified end-to-end for the candidate profile.

### Authority to resolve

Per [COMPATIBILITY_LEDGER.md](./COMPATIBILITY_LEDGER.md) “Required fixture asset integrity” and [UPSTREAM_REFRESH_POLICY.md](./UPSTREAM_REFRESH_POLICY.md) fixture CI policy:

1. Immutable release URL and exact asset name.
2. Byte length and lowercase SHA-256 after download, before extraction.
3. Extracted manifest SHA-256 and case counts.
4. Generation command with pinned Python/`uv` and crypto scheme.
5. Statement of no local modifications.

Checklist §14 “Fixture provenance”, §1 “Protocol release identity”.

### Blocking impact

Compatibility fingerprint, CI gates, conformance claims, and release evidence. A tag name, ETag, or `latest` redirect is insufficient.

---

## OSD-009 — leanMetrics revision TBD

**Status:** unresolved  
**Owner phase:** P0 for pin; P06 observability for implementation  
**Blocked phases:** [06-observability](../06-observability/), P09 validator/node duty metrics, release acceptance

### Observed state

- Local pq-devnet-4 notes record **leanMetrics as TBD**.
- Peers expose metrics with similar themes (slot, head, finality, gossip, sync, proof timing) but **no shared pinned leanMetrics revision** appears across the baseline snapshots.
- [METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md) requires pinned leanMetrics names when available; otherwise `ethean_` prefixes with schema versioning.

### Authority to resolve

Exact `leanMetrics` commit or release named by the selected pq-devnet plan and any Hive/interop log contract for the same generation. Record in ledger toolchain/observability fields when added.

Checklist §12 “Metrics names/labels”, §12 “Structured logs/events”.

### Blocking impact

Dashboard compatibility, interop log correlation, release acceptance, and CI cardinality gates. Non-consensus local metrics may proceed only where leanMetrics is silent, but release claims of observability parity remain blocked.

---

## Additional blocking decisions (secondary register)

These remain open and can block P3/P4 but are not the nine primary gates above. Full detail lived in the 2026-09-19 audit; track in ledger fields:

| Topic | Ledger / checklist anchor | Owner |
| --- | --- | --- |
| Gossip message-ID preimage order | `network.observed_message_id_conflict`; checklist §9 | P0 / P3 |
| Fork / network identity bytes | `network.observed_fork_identifiers`; checklist §9 topics | P0 |
| Discovery: static ENR vs discv5 vs mDNS | `network.observed_discovery`; checklist §10 | P3 |
| Checkpoint trust model | `checkpoint.*`; checklist §11 | P0 / P3 |
| Stateful signer durability | `signer.*`; checklist §6 | P0 / P07 |
| Toolchain reproducibility | `toolchain.*`; checklist §15 | P0 |

## Closure record

Closing any decision requires a dated appendix entry with:

- selected value;
- authoritative source commit and path;
- fixture asset SHA-256 and case IDs;
- delegated crypto/network dependency commits;
- compatibility-ledger paths updated;
- positive and negative test evidence;
- recomputed compatibility fingerprint;
- owner-phase sign-off.

A decision **reopens automatically** when [UPSTREAM_REFRESH_POLICY.md](./UPSTREAM_REFRESH_POLICY.md) shows a semantic diff in any cited source, generated byte, rejection result, dependency revision, or security assumption.

## Non-negotiable rule

No phase may treat an observed peer value as the selected profile. Unresolved rows are hard gates, not placeholders. See [AUTHORITY_POLICY.md](./AUTHORITY_POLICY.md) and the checklist introduction: “Never resolve a disagreement by majority vote among clients.”
