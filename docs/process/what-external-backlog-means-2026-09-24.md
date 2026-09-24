# What “external” means on the Ethean backlog (2026-09-24)

## Short answer

**External** = work that is **not owned by this repo’s code**. We keep the
client ready to consume it, but we **must not invent** the missing values or
pretend upstream published them.

## Three external buckets

| Bucket | What it is | Why Ethean cannot “just finish” it |
| --- | --- | --- |
| **A2 fork digest** | Hex digest that isolates gossip topics for a live pq-devnet mesh | Published per interop session / operator paste. Inventing one would put us on a private topic peers never join. |
| **A3 bootnodes** | QUIC multiaddrs of live mesh entry nodes | Same: session-specific. Empty files under `config/networks/` are intentional placeholders. |
| **Fixture re-fill** | Regenerated leanSpec JSON dumps (`uv run fill`) so `at_9` / `dead_9` carry full attestation bodies | Owned by **leanEthereum/leanSpec**. We already gate empty-body asserts; re-enabling weight checks needs upstream fill, then we re-import fixtures. |
| **Hive matrix row** | `ethereum/hive` `clients/ethean` + lean-devnets client list | Lives in **other GitHub orgs**. We ship the drop-in under `docker/hive/`; registration is an external PR. |

## What we *do* in-repo instead

- Wire CLI/env/file slots (`--fork-digest`, `--bootnodes`, empty config files).
- Fail-closed / warn when mesh risk is high without a digest.
- Keep Lean HTTP + FC + duties progressing without those pins.
- Document paste targets when values appear on leanroadmap / operator channels.

## Rule

Never invent public digests, bootnodes, or Hive registration claims. Continue
engineering on paths that compile and test without them.
