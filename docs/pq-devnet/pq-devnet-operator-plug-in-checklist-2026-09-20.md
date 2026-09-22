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
| Prover | `ETHEAN_PROVER_BIN` → `ethean-prover` beside `ethean` | Built with the node (`-p ethean-prover`) |
| XMSS | native, always on | Registry keys via `--validator-registry` |
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

## Signatures and proofs

XMSS signing and verification are native. Aggregate proofs use leanMultisig at
leanVM `e2592df4` (the pq-devnet-4 pin): every node verifies in-process;
aggregators and proposers also need the prover process.

```bash
cargo build --release -p ethean -p ethean-prover
ethean validator   # prints the XMSS, verifier and prover status
```

Hive: the image bundles `ethean-prover`; `HIVE_ETHEAN_PROVER_BIN` overrides the
path (the entrypoint exports `ETHEAN_PROVER_BIN`).

## Hive registration

1. Build `ethpandaops/ethean:local`.
2. Copy `docker/hive/upstream-clients-ethean/` → `ethereum/hive/clients/ethean/`.
3. Append client row + `ethean=devnet4,devnet5` in lean-devnets.
4. Run lean simulator with `--client ethean`.

Details: [hive-upstream-clients-ethean-dropin-2026-09-20.md](../hive-testing/hive-upstream-clients-ethean-dropin-2026-09-20.md).

## After paste: short soak

1. Boot shows `operator plug-in status` with bootnodes > 0 and no mesh-isolation warn.
2. Status handshake + range/root sync.
3. Local attest → `SignedAttestation` gossip → aggregator Type-1 prove → aggregation gossip.
4. Peer blocks import only after their Type-2 proof verifies (`gossip block rejected` logs say why).

## Related

- [deployment.md](../../deployment.md) — network tables and run recipes
- [leanmultisig-aggregation-2026-09-23.md](../lean-crypto/leanmultisig-aggregation-2026-09-23.md)
- [b1-leansig-vendor-backend-compile-2026-09-20.md](../lean-crypto/b1-leansig-vendor-backend-compile-2026-09-20.md)
- [hive-leansig-cargo-features-2026-09-20.md](../hive-testing/hive-leansig-cargo-features-2026-09-20.md)
