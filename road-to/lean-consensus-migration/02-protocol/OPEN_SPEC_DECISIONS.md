# Open Specification Decisions

## Purpose

This register lists protocol decisions that **block migration phases** until they are closed with authoritative evidence. Peer implementations supply observations only; they do not select values.

Primary evidence inputs:

- [COMPATIBILITY_LEDGER.md](./COMPATIBILITY_LEDGER.md) — machine-readable profile fields, observed alternatives, and peer snapshot commits.
- Phase 00 lock: [`../../../spec/pins/phase-00.lock.toml`](../../../spec/pins/phase-00.lock.toml)
- Protocol surface: [`../../../spec/pins/protocol-surface.toml`](../../../spec/pins/protocol-surface.toml)
- Fixture manifest: [`../../../spec/fixtures/phase-00/manifest.toml`](../../../spec/fixtures/phase-00/manifest.toml)

**Authority pin:** `leanEthereum/leanSpec@0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` (tree `4a0a358abf0447c26ad64ad6e57fd1e59ecccafd`).

Closure for OSD-001..OSD-008 was recorded on 2026-09-19 against that commit and the production fixture digest. **OSD-009 remains unresolved.**

## Decision summary

| ID | Decision | Selected value | Authority | Blocked phases (if reopen) | Status |
| --- | --- | --- | --- | --- | --- |
| OSD-001 | `MAX_ATTESTATIONS_DATA` | **8** | leanSpec `lstar/config.py` + lock | P03, P05, P08, P09, P10 | **resolved** |
| OSD-002 | XMSS public-key width | **52** bytes (`Bytes52`) | leanSpec XMSS + validator containers | P03, P07, P04 | **resolved** |
| OSD-003 | XMSS signature width | **2536** bytes | `PROD_CONFIG.SIGNATURE_LENGTH_BYTES` | P03, P07, P08, P10 | **resolved** |
| OSD-004 | Crypto stack | leanSpec-internal XMSS + `lean-multisig-py` v0.0.9 | leanSpec crypto + pyproject pin | P03, P07, P08, P11 | **resolved** |
| OSD-005 | Block proof envelope | **Type-2** `MultiMessageAggregate` | `SignedBlock.proof` | P03, P05, P08, P09, P10 | **resolved** |
| OSD-006 | Fork choice / finality | **modified 3SF-era `lstar`** (not Goldfish/PQ heartbeat) | `forks/lstar/` | P05, P06, P09, P11 | **resolved** |
| OSD-007 | Snappy profile | gossip **raw**; req/resp **framed** | leanSpec networking + snappy modules | P10, P11 | **resolved** |
| OSD-008 | Fixture release identity | asset digest + size + generator commit | fixtures release + manifest | P00–P11, P4 | **resolved** |
| OSD-009 | Observability contract | leanMetrics TBD | — | P06 observability, P09, release | **unresolved** |

Owner phases follow [02-protocol/README.md](./README.md). Peer evidence commits remain in `COMPATIBILITY_LEDGER.md` under `peer_evidence`.

---

## OSD-001 — `MAX_ATTESTATIONS_DATA`: 8 vs 16

**Status:** resolved (2026-09-19)  
**Selected value:** `8` (`Uint8(8)`), name **`MAX_ATTESTATIONS_DATA`** (plural ATTESTATIONS)  
**Owner phase:** P0 — Protocol Lock  

### Resolution citation

- Path: `src/lean_spec/spec/forks/lstar/config.py` — `MAX_ATTESTATIONS_DATA: Final = Uint8(8)`
- Lock: `spec/pins/phase-00.lock.toml` → `[resolved.consensus].max_attestations_data`
- Surface: `spec/pins/protocol-surface.toml` → `[registry].max_attestations_data`

The historical “16” observations (Qlean-mini, some pq-devnet-4 notes) are rejected for this profile. Peer majority was never the authority.

### Observed alternatives (historical)

| Value | Peer evidence | Note |
| --- | --- | --- |
| **8** | Ream, Zeam, ethlambda, Lantern, gean | Matches selected pin |
| **16** | Qlean-mini; some pq-devnet-4 objectives | Not selected |

### Blocking impact if reopened

Block body SSZ bounds, transition validation, proposer gathering, aggregate pool sizing, gossip decode budgets.

---

## OSD-002 — Public-key width: 32 vs 52 bytes

**Status:** resolved (2026-09-19)  
**Selected value:** **52** bytes — SSZ type `Bytes52`  
**Owner phase:** P0  

### Resolution citation

- `src/lean_spec/spec/ssz_types.py` — `class Bytes52` with `LENGTH = 52`
- `src/lean_spec/spec/forks/lstar/containers/validator.py` — attestation/proposal keys as `Bytes52`
- Lock: `[resolved.crypto].public_key_bytes = 52`

Rejected for this profile: leanVM-internal 32-byte V=42 keys (gean evidence generation).

---

## OSD-003 — Signature width: 1,208 vs 2,536 bytes

**Status:** resolved (2026-09-19)  
**Selected value:** **2536** bytes  
**Owner phase:** P0  

### Resolution citation

- `src/lean_spec/spec/crypto/xmss/constants.py` — `PROD_CONFIG` (`LOG_LIFETIME=32`, `DIMENSION=46`, `BASE=8`) and `SIGNATURE_LENGTH_BYTES` property
- Computed length for `PROD_CONFIG` equals 2536
- Lock: `[resolved.crypto].signature_bytes = 2536`

Rejected: 1208-byte Poseidon2/V=42 and historical Peam 3112-byte generations.

---

## OSD-004 — Standalone leanSig vs leanVM internalized XMSS

**Status:** resolved (2026-09-19)  
**Selected stack:** **leanSpec-internal XMSS** (Poseidon / KoalaBear `PROD_CONFIG`) plus aggregation via **`lean-multisig-py` tag `v0.0.9`**  
**Owner phase:** P0  

### Resolution citation

- XMSS: `src/lean_spec/spec/crypto/xmss/`
- Dependency: leanSpec `pyproject.toml` — `lean-multisig-py>=0.0.9` sourced from `https://github.com/anshalshukla/leanMultisig-py` tag `v0.0.9`
- Annotated tag object: `6f79724eed9a4b33690cb6d52cbe60fb455d8770`
- Underlying commit: `39b9c397fd4d2c358f8efe2815d4fb78f0e47e67`
- Lock: `[resolved.aggregation]`

This is not the historical Peam Dim64 stack and not gean’s 32-byte leanVM-only profile.

---

## OSD-005 — Type-1 vs Type-2 block proofs

**Status:** resolved (2026-09-19)  
**Selected envelope:** **Type-2** — `SignedBlock.proof: MultiMessageAggregate` covering attestations and proposer  
**Owner phase:** P0  

### Resolution citation

- `src/lean_spec/spec/forks/lstar/containers/block.py` — `SignedBlock`
- `src/lean_spec/spec/forks/lstar/containers/aggregation.py` — `MultiMessageAggregate`
- Lock: `[resolved.block_envelope]`

Rejected for this profile: Type-1 per-message proof lists (Qlean-era evidence) and older single-message Peam shapes.

---

## OSD-006 — Modified 3SF-mini vs PQ heartbeat / Goldfish

**Status:** resolved (2026-09-19)  
**Selected generation:** **`lstar` / LstarSpec** — modified 3SF-era fork choice and finality  
**Not selected:** Goldfish / PQ heartbeat (roadmap direction only)  
**Owner phase:** P0  

### Resolution citation

- Fork tree: `src/lean_spec/spec/forks/lstar/` (`fork_choice.py`, `state_transition.py`, `config.py`, …)
- Lock: `[resolved.lean_spec].fork = "lstar"`, `fork_choice_generation = "modified_3sf_era"`

Beacon Casper FFG, epoch checkpoints, and 32-slot epoch mapping remain forbidden fallbacks.

---

## OSD-007 — Snappy: raw gossip vs framed request/response

**Status:** resolved (2026-09-19)  
**Selected profile:** gossip uses **raw Snappy**; req/resp uses **framed Snappy**; **both required** on their surfaces  
**Owner phase:** P0 (profile); P3 implementation  

### Resolution citation

- Gossip: `src/lean_spec/node/networking/gossipsub/behavior.py` — `snappy_raw_decompress`
- Req/resp: `src/lean_spec/node/networking/client/reqresp_client.py` — framed Snappy layout
- Modules: `src/lean_spec/node/snappy/` (`decompress`, `framing`)
- Lock: `[resolved.network]`

Secondary open items (message-ID preimage order, fork identifier bytes) remain in the secondary register; they do not reopen the raw-vs-framed decision.

---

## OSD-008 — Fixture release SHA-256

**Status:** resolved (2026-09-19)  
**Selected asset:** `fixtures-prod-scheme.tar.gz`  

| Field | Value |
| --- | --- |
| Release name | Latest production fixtures |
| Tag (locator only) | `latest` |
| `target_commitish` | `0b7d33ecbc9ee2435759c92de4da4d08d7faf1c8` |
| Size | 154123448 |
| SHA-256 | `21d9de7056b4e658031dc09e50c0e9dc1b0206089253258ce69ecff01154d4bd` |

### Resolution citation

- GitHub release metadata for tag `latest` (immutable pin is digest + size + generator commit)
- Manifest: `spec/fixtures/phase-00/manifest.toml`
- Lock: `[resolved.fixtures]`

Tarball is **not** committed to git. CI must verify digest before extraction.

---

## OSD-009 — leanMetrics revision TBD

**Status:** unresolved  
**Owner phase:** P0 for pin; P06 observability for implementation  
**Blocked phases:** [06-observability](../06-observability/), P09 validator/node duty metrics, release acceptance

### Observed state

- No separate **leanMetrics** git repository pin found for this profile.
- leanSpec exposes metrics via **`prometheus-client>=0.21.0,<1`** (`pyproject.toml`, `src/lean_spec/node/metrics/`).
- Until an upstream leanMetrics pin exists, Ethean may use an `ethean_` metric namespace with schema versioning (see [METRICS_CONTRACT.md](../06-observability/METRICS_CONTRACT.md)), but release claims of leanMetrics parity remain blocked.

### Authority to resolve

Exact `leanMetrics` commit or release named by the selected pq-devnet plan / Hive contract. Record in ledger when available.

Lock: `[unresolved.observability]`

---

## Additional blocking decisions (secondary register)

These remain open and can block P3/P4 but are not the nine primary gates above:

| Topic | Ledger / checklist anchor | Owner |
| --- | --- | --- |
| Gossip message-ID preimage order | `network.observed_message_id_conflict` | P0 / P3 |
| Fork / network identity bytes | `network.observed_fork_identifiers` | P0 |
| Discovery: static ENR vs discv5 vs mDNS | `network.observed_discovery` | P3 |
| Checkpoint trust model | `checkpoint.*` | P0 / P3 |
| Stateful signer durability | `signer.*` | P0 / P07 |
| Toolchain reproducibility (Rust version) | `toolchain.*` — **TBD**, `rustc` missing on Phase 00 host | P0 / P01 |

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

No phase may treat an observed peer value as the selected profile. Unresolved rows are hard gates, not placeholders. See [AUTHORITY_POLICY.md](./AUTHORITY_POLICY.md).
