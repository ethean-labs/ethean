# ethean-crypto

Native leanSpec post-quantum signatures for Ethean, plus the Type-1 / Type-2
aggregation surface. No external cryptography crates: the KoalaBear field,
Poseidon1 permutation, Keccak/SHAKE128 PRF and the generalized XMSS scheme
live in this crate and `#![forbid(unsafe_code)]` applies to all of it.

## XMSS (leanSpec `PROD_CONFIG`)

| Item | Value |
| --- | --- |
| Field | KoalaBear, `p = 2^31 - 2^24 + 1`, canonical `u32` LE on the wire |
| Hash | Poseidon1 (Hades), widths 16 and 24, `x^3`, 8 full + 20 / 23 partial rounds |
| PRF | SHAKE128, 16 bytes big-endian per field element |
| Encoding | aborting hypercube message hash, target sum 200, 46 chains, base 8 |
| Lifetime | `2^32` epochs; bottom trees of 65536 leaves, two prepared at a time |
| Public key | 52 bytes (`root[8] || parameter[5]`) |
| Signature | 2536 bytes (`offset, rho[7], offset, opening[32], hashes[46]`) |

Module map (`src/`):

- `field.rs` — `Fp` arithmetic, base-`p` limb decomposition, lazy reduction.
- `keccak.rs` — Keccak-f[1600] and SHAKE128 (FIPS 202 vectors in tests).
- `poseidon/` — round constants (`rc16.rs`, `rc24.rs`), circulant MDS,
  permutation, compression / domain-separator / replacement-sponge modes.
- `xmss/native/` — parameters (`PROD`, `TEST`), tweaks, PRF, tweakable hash,
  encoding, Merkle top/bottom trees, SSZ codecs, `key_gen` / `sign` /
  `verify` / `advance_preparation`.
- `backend.rs` — `ProductionBackend` (native PROD XMSS) and
  `SecretKeyMaterial` (decoded once, shared preparation window).
- `batch.rs` — `verify_batch` (all cores) and `PublicKeyCache`.
- `backend_test_hmac.rs` — HMAC stand-in with PROD wire sizes (`test-hmac`).

## What pins correctness

- Poseidon width-16 / width-24 permutation, compression, domain separator,
  sponge and chain-step vectors from leanSpec `tests/spec/crypto`.
- `testdata/xmss_test_keys/*.json`: leanSpec-generated TEST_CONFIG key pairs.
  Recomputing every bottom-tree leaf from the stored PRF key reproduces the
  stored nodes byte for byte, and signatures round-trip through SSZ.
- `leansig-test-keys/prod_scheme/0.json` (when checked out next to the
  workspace): 8 MB PROD secret key decodes, signs at several epochs and
  verifies against the published 52-byte public key.

Run `cargo test -p ethean-crypto`; add `--release -- timing_probe --nocapture`
for permutation and verify timings.

## Aggregation

- Max proof **524288** bytes, `LOG_INV_RATE=2`, leanVM pin `e2592df4…`.
- `verify_type2` needs a leanVM prover (feature `leanvm-backend` plus IPC).
  Without one it returns `BackendUnavailable` and callers fail closed.
- `test-aggregate` produces statement-bound synthetic proofs for unit tests
  and local smoke only. It is **not** a default feature; enabling it in a
  node build makes keyless proofs verify.

## Features

| Feature | Default | Meaning |
| --- | --- | --- |
| `test-hmac` | yes | `TestHmacBackend` for fast unit tests |
| `test-aggregate` | no | synthetic aggregate proofs (tests / smoke only) |
| `leanvm-backend` | no | leanVM prove/verify surface (fail-closed stub) |
| `leansig-backend` | no | no-op kept for feature forwarding; XMSS is always native |
| `serde` | no | serde derives on wire types |
