# Scrub Cursor trailers from unpublished commits (2026-09-19)

## Goal

Remove `Co-authored-by: Cursor <cursoragent@cursor.com>` (and related Cursor attribution lines) from commit messages that were **not yet on `origin/master`**, without changing code, authors, or already-pushed history.

## What changed

| Check | Result |
| --- | --- |
| Range | `origin/master..HEAD` only (625 commits) |
| Rewritten messages | 624 (1 identical prefix commit had nothing to strip) |
| Tip tree SHA | Unchanged (`6c840cd…`) |
| `origin/master` | Unchanged (`f8a3d3a…`) |
| Ahead count | Still 625 |
| Remaining Cursor trailers / `cursoragent@cursor.com` in that range | **0** |
| Author / committer | Still `Paro <pamenarti@gmail.com>` |
| Working tree vs pre-scrub tip | Empty diff |

Method: `git commit-tree` replay with the same trees and parent mapping; only the commit message bytes were filtered. No force push; remote history untouched.

## What was not changed

- Commits already reachable from `origin/master` (including ~25 that still have the trailer on the remote).
- File contents, `Cargo.lock`, docs bodies, etc.
- No push was performed.

## Local safety refs

- `backup/pre-cursor-trailer-scrub-2` — tip before this scrub (same tree as current `HEAD`).
- An earlier stash of mid-flight WIP was dropped after the scrub because that WIP had already been superseded by later commits on the branch; re-applying it would have conflicted with those commits.

## Follow-ups

- Keep `.githooks/commit-msg` and Cursor Attribution **off** so new commits stay clean.
- Do not delete `backup/pre-cursor-trailer-scrub-2` until you are satisfied after a normal (non-force) push of the cleaned tip.
