# Fork-choice tests the peers still trip on (2026-11-02)

Item 8 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

In-repo unit tests, same fixture digest. No leanSpec archive refill.

| Case | Where |
| --- | --- |
| Fresh votes stay pending at interval 0 without a proposal | `fork-choice` `interval_zero_without_a_proposal_leaves_votes_fresh` |
| Older vote does not replace a newer one | `an_older_vote_does_not_replace_a_newer_one` |
| Unknown head is not counted until the block exists | `a_vote_for_an_unknown_head_is_pending_until_the_block_exists` plus deferred replay after import |
| Tick in the past is rejected | `a_past_tick_is_rejected` |
| Prune drops votes at/below finalized, keeps live ones | `stale_votes_below_finalized_are_dropped_and_live_votes_stay` |
| Duplicate proposal after head advance / in-flight proof | `duty_propose_tests` |
| Stale sync duties stay gated | existing `ethean-sync` hysteresis tests |
