# External gates attempt: A2/A3, fixture re-fill, Hive matrix (2026-09-25)

## What we checked

### A2 fork digest / A3 bootnodes

| Source | Result |
| --- | --- |
| leanroadmap.org | Goals / clients only — **no** published digest or bootnode list |
| `config/networks/pq-devnet-{4,5}.{forkdigest,bootnodes}` | Comment-only placeholders (correct; do not invent) |
| Ream `lean_peers.yaml` | Empty by design (same operator-paste model) |

**Cannot finish in-repo.** Needs an active interop session paste.

### Fixture re-fill (`finalized_safety` / `at_9` / `dead_9`)

Owned by **leanEthereum/leanSpec** (`uv run fill`). Ethean already gates empty
wire bodies so FC suites stay green. Re-enabling weight asserts waits on
upstream regenerated JSON, then a fixture re-import here.

**Cannot finish without leanSpec fill + import.**

### Hive matrix

| Check | Result |
| --- | --- |
| `ethereum/hive` `clients/` | Has ream, zeam, qlean, ethlambda, gean, lantern, … — **no `ethean`** |
| In-repo drop-in | Ready: `docker/hive/upstream-clients-ethean/` |
| GH auth / local `./hive` | Missing on this machine — cannot open the upstream PR from here |

**What we shipped to make the matrix actionable:**

| Path | Role |
| --- | --- |
| `tools/hive/apply-ethean-client.sh` | Idempotent apply onto a hive checkout |
| `tools/hive/apply-ethean-client.ps1` | Windows wrapper |
| `tools/hive/PR_BODY.md` | ethereum/hive PR text |
| `tools/hive/README.md` | Operator pointer |

Operator (or `gh auth login`) still must:

1. Confirm `ghcr.io/ethean-labs/ethean:devnet5` exists.
2. Clone `ethereum/hive`, run the apply script, open the PR.
3. Run `./hive --sim lean --client ethean …`.

## Rule reminder

Never invent public digests, bootnodes, or claim Hive matrix merge before the
upstream PR lands.
