# Yield Type-1 to the early block proof (2026-11-02)

Item 3 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

At leanVM `e2592df4` a Type-1 is about 1.1 s and a two-component Type-2 about
3.3 s. The early block window is the last two 800 ms intervals (1.6 s). A
Type-1 started at interval 2 is still on the worker when interval 3 queues
the block.

When this node proposes slot N+1 and the aggregate pool already has a
verified Type-1, interval 2 does not start another Type-1. The next block
uses the pooled proof. If the pool is empty, Type-1 still runs so a single
aggregator can put a body on the next block.

Union merges were already skipped on that path. There is no leanVM job
cancel. This host did not re-time a 2-aggregator soak.
