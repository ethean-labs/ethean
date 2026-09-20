# Console log colors (2026-09-20)

Stdout dumps use an Ethean line format close to common Lean client logs
(`timestamp LEVEL target: message`) but with a green-forward palette:

| Piece | Color |
| --- | --- |
| Timestamp | dim |
| `INFO` | bright green (good-path / finality) |
| `WARN` | bright yellow |
| `ERROR` | bright red |
| Target on INFO | soft green (headers; not Ream yellow) |

`--data-dir` still tees a **plain** file under `log/ethean-*-log` (no ANSI).

Code: `bin/ethean/src/console_fmt.rs`, `bin/ethean/src/file_log.rs`.
