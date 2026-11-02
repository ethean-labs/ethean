# Borrow parent state on import (2026-11-02)

Item 1 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

`pre_state_for_parent` returned an owned `State`. `apply_block` already clones
the pre-state to produce the post-state, so every gossip/sync import and every
accepted own block paid for a second full copy (registry, historical hashes,
justification bits).

The helper now returns `&State`. After the STF borrow ends, gossip import
clones only `pre.validators` for Split recovery. Own-block accept copies the
parent header slot into a local before mutating the owner.
