# Auto-flush pending block gossip on wall ticks (2026-09-19)

## Hook

- `run_wall_duty_loop` and `run_until_signal` take an `after_step` callback
- After each `apply_wall_step`, the hook may emit one extra `ChainEvent`

## Client

- `EtheanClient::run_wall_with_flush` / `run_until_signal_with_flush` live in `client_swarm`
- With `libp2p-quic`: hook calls `swarm_pump::flush_pending_event`
- Without the feature: hook is a no-op (`Ok(None)`)

## Events

- Successful flush → `ChainEvent::ProposalPublished { topic, payload_len, has_type2_proof }`

## Verification

```text
cargo test -p ethean-node --lib wall_loop
cargo test -p ethean-node --features libp2p-quic --lib
cargo test -p ethean-node --lib
```

## Still open

- XMSS/proposer signing into the Type-2 envelope (pool proof remains best-effort)
- leanVM FFI so non-empty proofs verify on ingest
- leanSig clean git dependency
