# ATTESTATION_COMMITTEE_COUNT on profile and gossip subnets (2026-09-20)

## Goal

Wire Hive / leanSpec `config.yaml` `ATTESTATION_COMMITTEE_COUNT` into the live
`ChainProfile` and use that value for attestation subnet selection and QuicSwarm
gossip subscriptions (instead of a hard-coded `% 64` or smoke-only 4).

## Changes

### Profile

- `ChainProfile::with_attestation_committee_count` overrides ACC and re-validates.
- `ChainProfile::attestation_subnet_count` clamps ACC to `u16` (minimum 1).

### Binary (`ethean start --lean-config`)

- After loading `config.yaml`, `lstar_devnet()` is adjusted with
  `lean.attestation_committee_count.max(1)` before `EtheanClient::with_genesis`.

### Duties

- Local attester subnet: `validator_index % profile.attestation_committee_count`
  (fallback 1 when profile missing).
- Aggregator provisional subnet: message-root mod profile subnet count (no longer
  hard-wired to `SMOKE_ATTESTATION_SUBNETS`).

### Network boot

- `prepare_boot_network` / `QuicSwarm::bind_for_fork_segment_subnets` take an
  explicit subnet count from `client.profile.attestation_subnet_count()`.
- Default fork-segment bind still uses `SMOKE_ATTESTATION_SUBNETS` (4) when the
  caller does not pass a count.

## Operator notes

- Hive fixtures with `ATTESTATION_COMMITTEE_COUNT: 1` subscribe to
  `attestation_0` only — matching typical pq-devnet leanSpec pins.
- Raising ACC above 4 expands the subscribed mesh; peers must agree on the same
  count for subnet gossip to meet.

## Follow-ups

- leanSpec committee → subnet assignment (replace provisional root-hash mapping).
- Upstream Hive `clients/ethean` pin once image builds ship this binary.
