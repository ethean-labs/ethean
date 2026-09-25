# ReamLabs leanstart + lean-spec-tests indexed (2026-09-25)

## Goal

Treat [ReamLabs/leanstart](https://github.com/ReamLabs/leanstart) and
[ReamLabs/lean-spec-tests](https://github.com/ReamLabs/lean-spec-tests) as
standing references for Lean Consensus work — beside peer clients and
leanEthereum official repos — without vendoring either tree into git.

## Local research (gitignored)

Deep notes: `bazalinacaklar/reamlabs-tooling/`

| Note | Repo |
| --- | --- |
| `leanstart.md` | Kind/Helm multi-client orchestrator (genesis, keys, metrics, subnets) |
| `lean-spec-tests.md` | Shared leanSpec prod-scheme vectors + suite map |
| `README.md` | Index + authority order |

## What each is for

| Repo | Ethean use |
| --- | --- |
| leanstart | Operator-shaped local (Kind) or remote (k3s) multi-client mesh; artifact layout peers expect; path to register an Ethean image later |
| lean-spec-tests | Secondary fixture source for FC / STF / SSZ / proofs / networking / API; `keys/prod_scheme/` for matching key material |

## Authority order (unchanged)

1. leanroadmap generation card + leanEthereum/pm plan
2. leanSpec pin + in-repo fixture digest (`spec/fixtures/phase-00/manifest.toml`)
3. lean-spec-tests as shared dump / cross-check
4. leanstart as orchestration reference (not protocol)
5. Peer clients for interop behavior only

## Pins recorded (2026-09-25 skim)

- leanstart `master` @ `869904c` (multi-host / injected genesis tooling)
- lean-spec-tests `master` @ `d92f9e4` (devnet5 vectors; leanSpec
  `e8014f9c5f384f7593e4ca819c0ba422a7754754`)

Re-check image tags and commits before claiming mesh interop; leanstart README
client images were still **devnet4**-tagged at skim time.

## Gaps for Ethean

- No `ethean` client key in leanstart yet (needs published image + Helm/CLI
  mounts matching peer pods).
- HTTP port convention: leanstart documents `:5055`; Ethean Hive lean HTTP uses
  `:5052` / `/lean/v0` — document when registering.
- Fixture refresh must keep manifest SHA as gate; do not replace with an
  unpinned lean-spec-tests tip.

## Do not

Clone either repo into the Ethean git tree. Summaries stay in `docs/`; extracts
stay under `bazalinacaklar/`.
