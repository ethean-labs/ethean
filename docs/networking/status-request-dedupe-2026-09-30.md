# Status requests sent once per session (2026-09-30)

## Problem

Every local devnet start logged 4–5 `Status handshake failed ... no pending
status for peer` lines per node, next to the 3 completed handshakes.

`prepare_status_outbounds` built a request for every peer in the session book
on every network pump window. The book only knew "pending" (waiting for a
reply), not "already sent", so a reply slower than one pump window caused a
second and third request. The first reply completed the session; the later
replies found no session and failed. Each extra request also left an entry in
the request tracker.

## Change

- `StatusSessionBook` (`crates/network/src/reqresp/status_session.rs`) records
  when the request to each pending peer went out (`mark_sent`).
  `peers_due(now)` returns pending peers with no request in flight, or with one
  older than `STATUS_RESEND_AFTER` (10 s). The pump has no event for a failed
  outbound request, so the timed resend is the recovery path.
- Disconnect and a completed reply clear the send time; a reconnect starts a
  fresh session. A duplicate connection while a request is in flight does not
  trigger another send.
- `prepare_status_outbounds` takes the book mutably and sends only to due peers.

## Check

4 nodes × 3 validators, 300 s, node 3 joining at +90 s from a checkpoint,
node 2 killed at +190 s and restarted 20 s later:

| | Before | After |
| --- | --- | --- |
| `no pending status` failures per node | 4–5 | 0 |
| Completed handshakes | one per peer | one per peer (joiner and restarted node included) |

The joiner reached the mesh head within one 15 s poll. The restarted node was
back at the head on the next poll. Finalized reached slot 61 at head 75.

Tests: `sent_request_is_not_repeated_until_it_times_out`,
`prepares_one_outbound_per_pending_peer` (second call sends nothing).
