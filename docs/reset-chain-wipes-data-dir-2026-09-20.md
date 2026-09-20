# --reset-chain empties data-dir (2026-09-20)

`--reset-chain` now deletes **everything** under `--data-dir`, including
`log/ethean-*-log`, `blocks/`, SSZ, `ethean.redb`, and leftover files.

The directory itself stays. Wipe runs **before** the new process log is opened,
so this start still writes a fresh `log/ethean-YYYY-MM-DD-HHMMSS-log`.

`--ephemeral` or a missing `--data-dir` still ignores the flag.
