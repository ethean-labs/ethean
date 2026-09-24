# Remote switch to ethean-labs/ethean (2026-09-25)

## Why

The project org repo is now [ethean-labs/ethean](https://github.com/ethean-labs/ethean).
The earlier personal fork [pamenarti/ethean](https://github.com/pamenarti/ethean)
is no longer the push target for day-to-day work.

## What we did

1. Local `master` already matched `pamenarti/ethean` at **1977** commits
   (`0d992e2`, VERSION **0.1.62**).
2. `ethean-labs/ethean` was **23** commits behind (stopped at `0.1.60`).
3. Fast-forward push: `915d90d..0d992e2` → `ethean-labs` `master`.
4. Remotes on this clone:
   - `origin` → `https://github.com/ethean-labs/ethean.git` (default push/pull)
   - `pamenarti` → old personal fork URL (fetch only; push URL `DISABLED`)

## Verify

```text
git remote -v
git rev-list --count HEAD
git rev-list --count origin/master
git status -sb
```

`origin` tracks [ethean-labs/ethean](https://github.com/ethean-labs/ethean).
After the catch-up push the labs repo matched the personal fork at **1977**;
the remote-switch docs then landed on labs only (labs ahead of `pamenarti`).
`pamenarti` push URL is `DISABLED` so accidental pushes fail.
