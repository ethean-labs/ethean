# Data-dir process logs (2026-09-20)

Durable `--data-dir` runs now tee tracing to:

`PATH/log/ethean-YYYY-MM-DD-HHMMSS-log`

UTC stamp, no colons (Windows-safe). Stdout still prints. `--ephemeral` (or no
`--data-dir`) stays console-only.

`--reset-chain` does not delete `log/`.

Wired in `bin/ethean/src/file_log.rs`; names live in
`crates/node/src/persist_paths.rs`.
