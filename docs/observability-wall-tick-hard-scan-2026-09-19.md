# Observability, wall ticks, and hard BLS scan (2026-09-19)

## Network wire

- `fork_identifier_bytes` / `fork_segment_hex` in `ethean-network-wire` (interim SHA-256 prefix)
- `fork_segment_from_name` delegates to the hex helper

## Node

- `wall_tick`: build `DutyTick` from `SlotClock` + `TimeSource`; interval delta helper (no sleep)
- `observability`: metrics registry, readiness bits, Lean `/lean/v1/health` dispatch smoke
- `EtheanClient::start` marks storage/crypto/signer, smokes health, records head/lag gauges

## Release

- `tools/release/legacy-scan.ps1` hard-fails on `blst` / `blstrs` / `bls12_381` under `crates/` and `bin/`
- Hard report: `CLEAN` after node BLS purge

## Verification

```text
cargo test -p ethean-node -p ethean-network-wire --lib
powershell -File tools/release/legacy-scan.ps1
```

ethean-node 16 tests, network-wire 12 tests; hard scan CLEAN.

## Still open

QUIC bind, RocksDB open implementation, continuous wall sleep loop in the binary, leanSig/leanVM FFI, soft-scan allowlist tightening for intentional `panro` refusal strings.
