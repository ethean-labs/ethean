# `just apitest` / Hive HTTP gaps (2026-11-02)

Item 9 of
[development-todo-full-backlog-2026-11-02.md](../process/development-todo-full-backlog-2026-11-02.md).

Historical miss: `GET /lean/v*/events` returned 501. It is a JSON poll
(`events`, `drained`, `pending`) or SSE when `Accept: text/event-stream`.
`:5052` integration now asserts 200, not 501.

Hive `rpc_compat` still only requires:

- `GET /lean/v0/health`
- `GET /lean/v0/checkpoints/justified`
- `GET /lean/v0/fork_choice`
- `GET /lean/v0/states/finalized`
- `GET /lean/v0/blocks/finalized`

Operator extras on the same port: ready, identity, events, `chain/head|finalized|sync`
on both v0 and v1. `GET /lean/v0/validator/duties` stays unmatched (v1 only).
`GET /metrics` on `:5052` stays a 404 pointer to `:9100`.

leanSpec `api_endpoint` still runs from the extracted fixture archive when
present (`api_fixture_tests.rs`). This host did not re-extract the archive.
