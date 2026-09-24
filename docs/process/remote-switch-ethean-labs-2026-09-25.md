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
   - `pamenarti` → old personal fork URL (optional fetch only; do not push)

## Verify

```text
git remote -v
git rev-list --count HEAD
git rev-list --count origin/master
git status -sb
```

Both counts should be **1977** and `master` should track `origin/master`.
