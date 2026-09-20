# ethean-spec-fixtures

Index and parse leanSpec filled consensus JSON fixtures. Maps known
`rejectionReason` strings onto `ethean-fork-choice::ForkChoiceError`, and runs
mapped fork-choice **rejection** steps (`create_store` + `on_block`).

Hive client registration and full tick/vote/STF runners are follow-ups — see
[`../../docs/hive-leanspec-fixture-consumer-2026-09-20.md`](../../docs/hive-testing/hive-leanspec-fixture-consumer-2026-09-20.md)
and
[`../../docs/leanspec-fc-rejection-runner-2026-09-20.md`](../../docs/lean-spec/leanspec-fc-rejection-runner-2026-09-20.md).
