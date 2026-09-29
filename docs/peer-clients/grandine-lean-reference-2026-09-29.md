# Grandine lean as a peer reference (2026-09-29)

Grandine's Lean client (`grandine_lean`, leanstart name `grandine`) joins the
reference list.

- Repo: https://github.com/grandinetech/lean (code in `lean_client/`)
- Reviewed: branch `devnet-5-leanvm-main` @ `87ca717d` (2026-08-21)
- Pins there: leanVM `a5909d18` (Ethean: `e2592df4`), Grandine libraries at
  `c4b676e3`, forked rust-libp2p `91e8931e`

## What we took from it

| Topic | Grandine lean | Effect on Ethean |
| --- | --- | --- |
| `/lean/v0/health` | Exactly `{"status":"healthy","service":"lean-rpc-api"}` | Ethean had added `version`, which failed the leanSpec API endpoint fixture. Removed; the version is still on the identity endpoint |
| Block timing | At interval 4 of slot N it builds and signs the block for N+1, then holds it until the slot starts | Same pattern now in Ethean (see [early-block-build-idle-pump-2026-09-29.md](../lean-crypto/early-block-build-idle-pump-2026-09-29.md)); ethlambda does the same |
| `LOG_INV_RATE` | Read from genesis `config.yaml` (default 2) | Ethean parses the key and warns when it differs from the built-in 2. Verification reads the rate from the proof, so it does not break verification |
| Proposer merge | `--enable-proposer-aggregation` is off by default: keep the best single proof per attestation data | Not adopted. Only worth revisiting if block proofs keep missing the slot |

Not reviewed yet: Status/sync peer choice, gossip scoring, storage.

Local deep notes live in `bazalinacaklar/peer-client-library/grandine-lean.md`
(gitignored).
