# B2 leanVM refuse_reason and D2 aggregator subnet substrate (2026-09-20)

## B2 honesty

- `LeanVmGate::refuse_reason()` distinguishes feature-off, FFI unlinked, and IPC-not-ready gaps.
- `client_boot::boot_gates` warns when dialing bootnodes without production leanVM (Type-2 verify stays fail-closed).
- Complements the earlier leanSig `refuse_reason` mesh warning (B1 honesty).

## D2 progress

### Gossip subnets

- `LeanGossipTopics` now carries `attestations: Vec<String>` for
  `SMOKE_ATTESTATION_SUBNETS` (4) topics: `attestation_0` .. `attestation_3`.
- QuicSwarm subscribe path uses `as_slice()` over block + aggregation + all smoke subnets.
- `attestation_0()` remains a compat accessor.

### Aggregator duty hook

- New `duty_aggregator::evaluate_aggregator_duties` walks the aggregate pool and calls
  `ethean_validator::run_aggregator`.
- Emits `ChainEvent::AggregatorReady { data_root, subnet, coverage }` when coverage clears.
- Provisional subnet = low bits of `message_root` mod smoke subnet count (until leanSpec
  committee → subnet mapping).
- Hooked from `duty_step::apply_wall_step` after the duty gate passes, before proposal plan.

## Still open

- Production leanVM prove/verify (real B2/B3 close).
- Dispatch Type-1 prove + aggregation gossip publish on `AggregatorReady`.
- Operator / leanSpec subnet count pin (may exceed smoke 4).
- Committee assignment table instead of provisional root hashing.

## Tests

- `cargo test -p ethean-crypto --lib default_build_reports_gaps`
- `cargo test -p ethean-network --lib topics`
- `cargo test -p ethean-node --lib duty_aggregator`
