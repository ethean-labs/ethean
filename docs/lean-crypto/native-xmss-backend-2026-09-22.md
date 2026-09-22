# Native leanSpec XMSS backend (2026-09-22)

## Why

`ethean-crypto` used to wrap the `leansig` git crate behind a
`leansig-backend` feature that did not build without a local `num-bigint`
vendor patch. In practice the production backend was never compiled, every
node ran the HMAC smoke backend, and registry keys were refused. The
default feature set also compiled `test-aggregate` into the shipped binary,
so `apply_block` accepted keyless synthetic Type-2 proofs.

## What changed

- `crates/crypto` now implements leanSpec XMSS itself: KoalaBear field,
  Poseidon1 (widths 16 / 24) with the leanSpec round constants and
  circulant MDS, SHAKE128 PRF, aborting hypercube message hash with
  target-sum encoding, top/bottom Merkle trees with the sliding
  two-bottom-tree preparation window, and SSZ codecs for public keys,
  signatures and secret keys. No external crypto crates, no `unsafe`.
- `ProductionBackend` is always available and uses the native PROD scheme.
  `SecretKeyMaterial` decodes an SSZ secret key once and shares the decoded
  key (and its preparation window) between clones; `prepare_for_epoch` can
  be called ahead of time from a background task.
- `verify_batch` fans independent verifications across all cores;
  `PublicKeyCache` decodes each validator's 52-byte key once.
- Registry loading (`crates/node/src/registry_keys.rs`) decodes the XMSS
  secret key, takes the activation window from the key, derives the public
  key and refuses a `pubkey_hex` that does not match it.
- `LocalProposer` / `LocalAttester` install registry keys on the native
  backend unconditionally. `prefer_production` still starts with the smoke
  key unless `ETHEAN_PRODUCTION_KEYGEN=1` (full PROD keygen builds two
  65536-leaf bottom trees at boot).
- `test-aggregate` is no longer a default feature. It is enabled for
  `ethean-node` dev-dependencies and the `ethean-leanvm-mock` binary, and
  exposed as an opt-in `test-aggregate` feature on `ethean` for offline
  smoke runs.
- `LeanSigGate` reports ready; the `leansig-backend` feature is kept as a
  no-op so existing run scripts keep building. The vendor patch, its
  PowerShell helpers and `.cargo/config.toml.example` are removed.

## Evidence

- leanSpec Poseidon vectors (`tests/spec/crypto/test_poseidon.py`,
  `tests/spec/crypto/xmss/test_poseidon.py`) pass as unit tests.
- leanSpec TEST_CONFIG key pairs (vendored under
  `crates/crypto/testdata/xmss_test_keys`): every bottom-tree leaf is
  reproduced from the stored PRF key; sign / verify / SSZ round-trip and
  tamper rejection pass.
- leanSig `prod_scheme/0.json` (8 MB secret key, lifetime `2^32`, 46
  chains): decode, sign at epochs 0, 1, 65535, 65536, 131071 and verify
  against the published public key (test skips when the checkout is
  absent).
- Release-mode timings (single thread): Poseidon24 about 8 µs, Poseidon16
  about 3 µs, TEST-scheme verify about 0.2 ms, PROD verify about 0.8 ms
  (`cargo test -p ethean-crypto --release -- prod_scheme --nocapture`);
  `verify_batch` scales across cores.

## Follow-ups

- Karatsuba / NTT circulant MDS to close the gap to Plonky3's SIMD path.
- Hook `verify_batch` into gossip attestation admission once the pool
  verifies individual `SignedAttestation` XMSS signatures.
- Background preparation task that advances the window before slot
  boundaries instead of on the signing path.
