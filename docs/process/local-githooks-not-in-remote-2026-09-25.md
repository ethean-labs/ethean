# Local `.githooks`, tracked `.gitignore` (2026-09-25)

## Goal

- On a developer machine (Cursor / Agent commits): keep a local `.githooks/` tree
  (e.g. `commit-msg` strip of Cursor attribution) and `core.hooksPath=.githooks`.
- On GitHub remotes (`origin`, `ethean-labs`, …): **do not** ship `.githooks/` —
  the directory must not appear in the tree.

## What stays in git

| Path | In remote? | Why |
| --- | --- | --- |
| `.gitignore` | **Yes** | Shared ignore rules (`target/`, `bazalinacaklar/`, `.cursor/`, …). |
| `.githooks/` | **No** | Local-only hooks; listed in `.gitignore` as `.githooks/`. |

`.gitignore` itself must remain tracked. Ignoring or deleting it from the remote
would drop shared excludes and would also stop enforcing “never push `.githooks`”
for other clones.

## Local setup (once per clone)

1. Ensure `.gitignore` contains a `.githooks/` line (already on `master`).
2. Recreate the hook files under `.githooks/` (they are not in the remote).
3. `git config core.hooksPath .githooks` (repo-local; do not commit this).

`git check-ignore -v .githooks/commit-msg` should point at `.gitignore`.

## Remote state after 2026-09-25

`master` already has the intended tree: `.githooks/` removed from the index, and
`.githooks/` present in `.gitignore`. Working copies keep the hook files as
untracked ignored paths.
