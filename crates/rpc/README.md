# ethean-rpc

Narrow Lean HTTP API (`/lean/v1/…`). **No** `/eth/v1/` Beacon compatibility.

- Public: health, ready, identity, head, finalized (with trust_source), sync,
  fork_choice, bounded duties
- Admin: shutdown + event poll (`GET /lean/v1/events` drains a redacted ring
  buffer; auth on non-loopback)
- Body/rate budgets in `limits.rs`
