# leanVM gate and leanSig proposer feature forwards (2026-09-19)

## leanVM

- `LeanVmLinkStatus` / `LeanVmGate` expose pin, Cargo feature, and `LEANVM_FFI_LINKED` separately
- `FfiStatus.leanvm` is true only when `LeanVmGate::ready()` (feature **and** FFI)
- Stub prove/verify still fail closed; if the FFI flag were flipped without symbols, a second error path refuses fake success
- Client boot logs `leanvm_feature`, `leanvm_ffi`, and `leanvm_pin`

## leanSig / LocalProposer

- `ethean-node` features: `leansig-backend`, `leanvm-backend` (forwarded to `ethean-crypto`)
- `LocalProposer::prefer_production` tries leanSig ProductionBackend when the feature is on, else test-hmac smoke
- `LocalProposer::is_production` reports which path loaded
- Default builds stay on test-hmac (no vendor patch required)

## Verification

```text
cargo test -p ethean-crypto --lib ffi_status
cargo test -p ethean-node --lib local_proposer
cargo test -p ethean-node --lib
# Optional (needs vendor patch): cargo test -p ethean-node --features leansig-backend
# Optional stub compile: cargo test -p ethean-node --features leanvm-backend
```

## Still open

- Upstream leanSig `num-bigint` 0.5 so git dep works without path override
- Set `LEANVM_FFI_LINKED` after real lean-multisig FFI symbols link
- Embed proposer XMSS bytes into the Type-2 proof blob when leanMultisig specifies the encoding
