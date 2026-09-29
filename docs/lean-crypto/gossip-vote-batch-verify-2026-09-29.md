# Gossip votes verified in parallel per pump window (2026-09-29)

## Problem

Every gossiped `SignedAttestation` was verified on its own, in arrival order, on
the chain-owner task. A PROD XMSS verify costs about 1 ms in release, so a burst
of subnet votes cost 1 ms per vote of serial work, and that work blocked the
duty loop. `ethean_crypto::verify_batch` (all cores) and `PublicKeyCache` already
existed, but nothing called them.

The network pump already hands over gossip in windows of 32–64 events
(`pump_swarm_budget` / `pump_swarm_window`). During the vote burst at interval 1,
those windows arrive full.

## What changed

- `gossip_attestation::preverify_votes(owner, &[(topic, payload)])`:
  - Decodes each attestation-subnet payload and runs the cheap checks:
    checkpoint order, the future-slot bound, and a registry key lookup.
  - Verifies the survivors with `verify_batch` on all cores.
  - Returns an index-aligned `Option<PreVerifiedVote>`, which holds the key it
    checked against, the verdict, and the amortized time.
  - Windows with fewer than two votes are left to the inline path.
- `gossip_ingest::ingest_accepted` computes these verdicts for the whole window,
  then admits payloads **one at a time in arrival order**. The aggregator
  signature pool and fork-choice `on_attestation_data` see the same sequence as
  before, so a vote that arrives after its head block in the window still finds
  that block.
- `on_signed_attestation_with` / `ingest_attestation_gossip_with` use a verdict
  only if it was made against the key the registry holds at admission time.
  Otherwise they verify again inline, so a verdict can never approve a signature
  under a different key.
- Metrics: the verification histogram records the amortized time per vote. The
  validation time includes that share.

Aggregates (`/aggregation/`) are unchanged: one leanVM proof per message.

## Measured

`timing_probe_prod_batch_verify` (ignored; release, 20 workers, 256 copies of
one real PROD signature):

| Path | Total | Per signature |
| --- | --- | --- |
| serial `native::verify` | 272 ms | 1.06 ms |
| `verify_batch` | 27.7 ms | 108 µs |

## Tests

- `a_window_of_votes_is_verified_up_front_and_index_aligned`
- `a_lone_vote_is_left_to_inline_verification`
- `a_verdict_only_counts_for_the_key_it_was_made_against`
- `a_window_of_forged_votes_never_reaches_the_pool`
- `aggregation_flow` (needs PROD test keys and a built prover): three real votes
  and one forgery go through one batch; three are admitted and the forgery is
  rejected.

```bash
cargo test -p ethean-node --features libp2p-quic --lib gossip_
cargo test --release -p ethean-crypto --lib timing_probe_prod_batch_verify -- --ignored --nocapture
```
