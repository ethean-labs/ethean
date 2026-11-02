# Fork digest: warn-only, no invented A2 pin (2026-11-02)

Item 10 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

Default gossip segment is leanSpec lstar `GOSSIP_DIGEST` `"12345678"`. It is
**not** SHA-256(`fork_name`) (`aa4c6403…`). `--fork-digest` / `ETHEAN_FORK_DIGEST`
/ a non-comment line in `config/networks/*.forkdigest` still wins.

Dialing pq-devnet bootnodes without that pin logs a warning. Blank or
comment-only digest files stay empty on purpose. Do not invent an A2 value.
