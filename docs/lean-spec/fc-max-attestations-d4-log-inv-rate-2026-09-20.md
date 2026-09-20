# FC MAX_ATTESTATIONS_DATA gate + D4 log_inv_rate range (2026-09-20)

## Goal

After indexing [leanEthereum/pm](https://github.com/leanEthereum/pm) pq-devnet-4,
keep Ethean’s structural gates aligned with the written plan without adopting
unmerged D5 drafts.

## What landed

### Fork choice

- `on_block` rejects bodies with more than `MAX_ATTESTATIONS_DATA` (8) distinct
  entries — `ForkChoiceError::TooManyAttestationData`
- Fixture rejection map: `TOO_MANY_ATTESTATION_DATA`

### Crypto

- Comment + assert: `LOG_INV_RATE` stays in D4 protocol range `1..=4` while the
  pinned value remains `2`

### Authority reminder

pm objectives text that mentions `MAX_ATTESTATION_DATA = 16` is **not** used;
leanSpec + D4 summary + Ethean locks stay at **8**.
