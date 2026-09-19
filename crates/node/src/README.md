# ethean-node sources

Lean Consensus node library modules. Production binary: `bin/ethean`.

Active modules: `aggregation`, `block_builder`, `chain_owner`, `cli`, `client`, `clock`,
`commands`, `crypto`, `events`, `network` (re-exports `ethean-network`), `shutdown`.

Prefer `ethean_primitives` and `ethean_profile` over hard-coded consensus constants.
Beacon-era trees under this crate were deleted after Lean crate cutovers.
