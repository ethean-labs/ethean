# No AI / Cursor git attribution (2026-09-19)

## Problem

Agent commits in this repo were getting an automatic trailer:

```text
Co-authored-by: Cursor <cursoragent@cursor.com>
```

That marks history as AI-authored. We do not want that in public or shared git history.

## Policy

- Author and committer stay the normal local git identity (`user.name` / `user.email`).
- Commit messages are plain English: why the change exists. No AI product names, no co-author trailers, no “Made with Cursor”.
- Pull request bodies must not advertise Cursor/agent authorship.

## Repo controls

- Local Cursor rules (gitignored): `11-no-ai-git-attribution.mdc`, plus updates to the git / human-code / commit-after-each-file rules. After every agent commit, the agent must inspect HEAD and amend-scrub if a Cursor trailer slipped in.
- Hook: [`.githooks/commit-msg`](../../.githooks/commit-msg) deletes common Cursor / `cursoragent@cursor.com` trailers from the commit message before the commit is finalized.

Enable the hook path once per clone:

```bash
git config core.hooksPath .githooks
```

## Cursor IDE setting (required)

Attribution is on by default. Turn it off:

1. **Cursor Settings → Agent → Attribution** (or on 3.11+: **Git & PRs → Attribution**)
2. Disable commit attribution and PR attribution

Docs: [https://cursor.com/help/integrations/git](https://cursor.com/help/integrations/git)

If you also use Cursor CLI, set in `~/.cursor/cli-config.json`:

```json
{
  "attribution": {
    "attributeCommitsToAgent": false,
    "attributePRsToAgent": false
  }
}
```

## Cloud agents

Hosted / cloud agent commits may still force a Cursor author identity; IDE toggles do not fully cover that path. Prefer local agent commits for this repository.

## Unpublished history scrub (2026-09-19)

All **unpublished** commits (`origin/master..HEAD`, 625 commits) were message-rewritten to drop Cursor trailers / `cursoragent@cursor.com`. Trees, authors (`Paro <pamenarti@gmail.com>`), dates, and file contents were kept. `origin/master` was not rewritten (about 25 older pushed commits may still carry the trailer; changing those would need a force push).

Safety ref (local only): `backup/pre-cursor-trailer-scrub-2` points at the pre-scrub tip. Details: [scrub-unpublished-cursor-trailers-2026-09-19.md](scrub-unpublished-cursor-trailers-2026-09-19.md).
