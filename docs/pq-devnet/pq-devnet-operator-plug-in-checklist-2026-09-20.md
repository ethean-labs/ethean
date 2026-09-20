# pq-devnet operator plug-in checklist (2026-09-20)

When ethereum Lean / pq-devnets publish live values, fill these slots. The client
already wires CLI/env/file; empty slots mean offline smoke, not a broken binary.

This is **not** a production-ready claim. Mock leanVM and missing bootnodes stay
fail-closed by design.

## Priority order

| Layer | Sources (first wins) | Paste target when published |
| --- | --- | --- |
| A2 fork digest | `--fork-digest` → `ETHEAN_FORK_DIGEST` → `config/networks/pq-devnet-{4,5}.forkdigest` | Operator hex (gossip mesh isolation) |
| A3 bootnodes | `--bootnodes` → `ETHEAN_BOOTNODES` → `config/networks/pq-devnet-{4,5}.bootnodes` | QUIC multiaddrs (comma-separated) |
| leanVM IPC | `ETHEAN_LEANVM_PROVER` (+ optional `ETHEAN_LEANVM_IPC_PROBE=1`) | Pin-trusted binary path |
| leanSig | build with `leansig-backend` (+ vendor bigint patch until upstream 0.5) | Feature-enabled image |
| Hive | `docker/hive/upstream-clients-ethean/` → ethereum/hive `clients/ethean/` | Matrix + image tag |

## A2 / A3 (mesh)

```bash
ethean start --until-signal --network pq-devnet-5 \
  --fork-digest '<published-hex>' \
  --bootnodes '<quic-multiaddr>,…'
```

Or edit the empty comment-only files under `config/networks/` and restart.
Boot log line `operator plug-in status` shows `bootnodes`, `fork_digest_source`,
and `mesh_isolation_risk`. Dialing bootnodes without an operator digest warns
that Lean topics will not match peers.

**Do not** invent public bootnodes or digests.

## leanVM (Type-1 / Type-2 / Split IPC)

```bash
export ETHEAN_LEANVM_PROVER=/path/to/pin-trusted-leanvm
export ETHEAN_LEANVM_IPC_PROBE=1   # optional: flip protocol_ready after round-trip
```

Hive: set `HIVE_LEANVM_PROVER` / `HIVE_LEANVM_IPC_PROBE` (entrypoint exports the
`ETHEAN_*` names). Mock (`ethean-leanvm-mock`) is for CI only — not a mesh claim.
Pin must match `LEANVM_REV`. Helper: `tools/release/check-leanvm-prover.ps1`.

## leanSig (XMSS)

Default builds stay fail-closed. When upstream `num-bigint` 0.5 lands (or with
the local vendor patch):

```bash
# local
tools/release/vendor-leansig-bigint-fix.ps1
tools/release/check-leansig-backend.ps1 -Test

# Hive image
docker build -f docker/hive/Dockerfile \
  --build-arg CARGO_FEATURES=leansig-backend \
  -t ethpandaops/ethean:local-leansig .
```

## Hive registration

1. Build `ethpandaops/ethean:local` (or leansig variant).
2. Copy `docker/hive/upstream-clients-ethean/` → `ethereum/hive/clients/ethean/`.
3. Append client row + `ethean=devnet4,devnet5` in lean-devnets.
4. Run lean simulator with `--client ethean`.

Details: [hive-upstream-clients-ethean-dropin-2026-09-20.md](../hive-testing/hive-upstream-clients-ethean-dropin-2026-09-20.md).

## After paste: short soak

1. Boot shows `operator plug-in status` with bootnodes > 0 and no mesh-isolation warn.
2. Status handshake + range/root sync.
3. Local attest → Type-1 prove (IPC or test-aggregate) → aggregation gossip.
4. Peer XMSS / Type-2 verify only when leanSig / leanVM gates report ready.

## Related

- [deployment.md](../../deployment.md) — network tables and run recipes
- [b2-leanvm-ipc-live-probe-2026-09-20.md](../lean-crypto/b2-leanvm-ipc-live-probe-2026-09-20.md)
- [b1-leansig-vendor-backend-compile-2026-09-20.md](../lean-crypto/b1-leansig-vendor-backend-compile-2026-09-20.md)
- [hive-leansig-cargo-features-2026-09-20.md](../hive-testing/hive-leansig-cargo-features-2026-09-20.md)
