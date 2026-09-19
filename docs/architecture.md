# Architecture (Lean Consensus)

Ethean is a **Lean Consensus** client workspace, not a Beacon Chain node.

## Crate map

| Crate | Role |
| --- | --- |
| `ethean-primitives` / `ethean-profile` | Slots, forks, pinned chain profile (`lstar`) |
| `ethean-ssz` / `ethean-types` | Canonical SSZ containers |
| `ethean-crypto` | XMSS / leanSig surface (fail-closed without FFI) |
| `ethean-genesis` / `ethean-transition` / `ethean-fork-choice` | Genesis, state transition, 3SF-mini |
| `ethean-validator` | 4s / 5-interval duties |
| `ethean-network` / `ethean-network-wire` | Gossip admission, req/resp, QUIC facade |
| `ethean-storage` / `ethean-sync` | Schema `ethean-lc-d5-v1`, sync gates |
| `ethean-rpc` / `ethean-metrics` | `/lean/v1`, `ethean_` metrics |
| `ethean-node` | Chain owner shell + client |
| `ethean` (`bin/ethean`) | CLI |

## Ownership

`ChainOwner` is the sole writer of head/sync flags. Workers read snapshots and send `ChainCommand`s.

## Open gates

QUIC transport, RocksDB backend, leanSig/leanVM FFI, full duty loop wiring.

Details: [../road-to/lean-consensus-migration/README.md](../road-to/lean-consensus-migration/README.md).
