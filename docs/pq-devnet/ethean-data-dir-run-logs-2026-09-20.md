# Data-dir process logs (2026-09-20)

Durable `--data-dir` runs now tee tracing to:

`PATH/log/ethean-YYYY-MM-DD-HHMMSS-log`

UTC stamp, no colons (Windows-safe). Stdout still prints with green-forward
ANSI colors (see [ethean-console-log-colors-2026-09-20.md](../observability/ethean-console-log-colors-2026-09-20.md));
the file copy is plain text. `--ephemeral` (or no
`--data-dir`) stays console-only.

`--reset-chain` empties `--data-dir`, including `log/`. A new dated log is
opened after the wipe for that start.

Wired in `bin/ethean/src/file_log.rs`; names live in
`crates/node/src/persist_paths.rs`.
