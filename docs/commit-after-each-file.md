# Commits during a prompt

Commit as soon as a source or docs file is done, in English, without waiting for the whole prompt. One coherent file (plus the `mod.rs` that wires it) per commit. Still never push unless asked. Cursor rules and `bazalinacaklar/` stay untracked.

After a **finished development update** (not every mid-prompt file), bump the
workspace version once with `scripts/bump-version.ps1` / `.sh` and commit
`VERSION` + `Cargo.toml`. See [versioning.md](./versioning.md).

Never leave Cursor / AI attribution on commits (`Co-authored-by: Cursor`, `cursoragent@cursor.com`, “Made with Cursor”). See [no-ai-git-attribution-2026-09-19.md](process/no-ai-git-attribution-2026-09-19.md).
