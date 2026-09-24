# Hive image optional leansig-backend feature (2026-09-20)

## What landed

- `bin/ethean` forwards `leansig-backend` and `leanvm-backend` features to
  `ethean-node` / `ethean-crypto`.
- `docker/hive/Dockerfile` accepts `CARGO_FEATURES` build-arg so Hive images can
  compile with `leansig-backend` when the build context includes the local
  leanSig num-bigint vendor `[patch]` (see
  [`b1-leansig-vendor-backend-compile-2026-09-20.md`](../lean-crypto/b1-leansig-vendor-backend-compile-2026-09-20.md)).

Default Hive image stays fail-closed (no leansig) so CI/docker builds do not
depend on gitignored vendor trees.

## Recipe

```bash
# Default (QUIC only)
docker build -f docker/hive/Dockerfile -t ghcr.io/ethean-labs/ethean:local .

# With leanSig (after tools/release/vendor-leansig-bigint-fix.ps1 + .cargo/config.toml)
docker build -f docker/hive/Dockerfile \
  --build-arg CARGO_FEATURES=leansig-backend \
  -t ghcr.io/ethean-labs/ethean:local-leansig .
```

## Still open

- Upstream leanSig `num-bigint` 0.5 so git dep works without `[patch]`
- Published `ghcr.io/ethean-labs/ethean` image with leansig enabled
- leanVM FFI link in the same image (`leanvm-backend`)
