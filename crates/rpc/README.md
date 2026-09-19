# ethean-rpc

Narrow Lean HTTP API (`/lean/v1/…`). **No** `/eth/v1/` Beacon compatibility.

- Public: health, ready, identity, head, finalized (with trust_source), sync, bounded duties
- Admin: shutdown + event stream (auth on non-loopback)
- Body/rate budgets in `limits.rs`
