# Mesh finality fixes from the local proving devnet (2026-09-29)

Running three Ethean nodes against each other
([local-proving-devnet-windows-2026-09-29.md](../pq-devnet/local-proving-devnet-windows-2026-09-29.md))
showed that a `config.yaml` genesis mesh never justified past genesis. Unit
tests and fixture drives passed because none of them started from a sealed
genesis head with a live prover and several proposers. Five fixes, in the order
they were found:

## 1. Fork-choice store never started on a `config.yaml` genesis

`with_genesis_store` calls `seal_genesis_head` (caches the genesis state root in
`latest_block_header.state_root`) before `try_init_fork_choice`. The anchor
rebuild then compared that cached root with the hash of the *sealed* state,
which can never match, and the store was skipped at debug level. Every mesh
node ran without a `ForkChoiceStore`: no LMD head, no safe target, no store
justification.

`chain_fc::genesis_anchor` now rebuilds the anchor from the unsealed state
(header `state_root` zero, as leanSpec genesis) and accepts a sealed header
whose cached root equals that state's root; the store is created from the
unsealed state. A failed rebuild is now a warning. Test:
`store_initializes_after_the_genesis_header_is_sealed`.

## 2. Votes did not use the leanSpec attestation target

The local attester voted `target = safe_target`. leanSpec
`get_attestation_target` (lstar `validator_duties.py` @ `0b7d33ec`) starts from
the head, walks back at most `JUSTIFICATION_LOOKBACK_SLOTS` while above
`max(safe_target slot, finalized slot)`, then walks back until the slot is
justifiable after the finalized slot. Without that last step most votes
targeted slots such as 7, 13, 22, 26, which the STF ignores, so blocks carried
votes that could never justify anything.

`ForkChoiceStore::attestation_target(lookback)` and `attestation_source()`
(`crates/fork-choice/src/head/target.rs`) implement the spec; the source is the
head state's justified checkpoint with the zero genesis root replaced by the
head root (this also removed the "unknown source block" drops before the first
block). The attester uses them whenever the store is live and skips a vote whose
target would fall behind its source.

## 3. Finished block proofs dropped when the head moved

`accept_block_proof` refused any proof whose parent was no longer the head.
With proofs taking several seconds, another node's late block almost always
arrived meanwhile, and the aggregator (node 0) never published a block. With a
live store the block now stays publishable while its parent is still in the
tree and no block at or past its slot is head; it is still the only block for
that slot and fork choice decides. Without a store the old rule stays. The
`HeadUpdated` event reports the store head after import, not the new block.

## 4. The block data cap filled up with votes that change nothing

`select_body` scans candidates in target-slot order, so the default cap of 3
(`--max-block-attestation-data`) kept the three oldest data, often already
counted on chain. `select_body_capped` applies the cap inside the scan and,
below the spec maximum, skips votes that leave the justification bookkeeping
unchanged (`latest_justified`, `latest_finalized`, `justified_slots`,
`justifications_*`). At the spec maximum the selection is unchanged. Tests:
`capped_selection_skips_votes_already_counted`,
`capped_plans_leave_out_votes_that_change_nothing`.

A debug line per proposal now lists the pool (`slot@source->target:coverage`),
the chosen votes and how many candidates each rule skipped.

## 5. Status retried forever after a simultaneous dial

Two nodes dialing each other open two QUIC connections; libp2p closes one.
`QuicSwarm` forgot the peer on any `ConnectionClosed`, even with
`num_established > 0`, so the surviving connection had no peer id mapping and
every Status request failed with "peer not connected" (about 2100 log lines per
150 s). The peer is now forgotten only when its last connection closes.

## Result

Same 3-node setup: justified 4 → 6 → 9 → 25 → 30, slot 25 finalized on all
nodes, 41 of 42 block proofs published, no Status retries.
