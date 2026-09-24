# Identity version/ready + Hive v0 ready alias (2026-09-25)

## Why

1. `/lean/v1/node/identity` only returned `network` + `peer_id`.
2. Ready lived only under `/lean/v1/ready` while health already aliases v0/v1.

## What landed

- Identity JSON includes `version` (`CARGO_PKG_VERSION`) and `ready`.
- `GET /lean/v0/ready` aliases the same ready handler as v1.
- `ApiSnapshot.ready` mirrors the readiness latch on each publish.

## Verify

```text
cargo test -p ethean-rpc --lib
curl -s http://127.0.0.1:5052/lean/v1/node/identity
curl -s -o NUL -w "%{http_code}" http://127.0.0.1:5052/lean/v0/ready
```
