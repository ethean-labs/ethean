# Data Flow and State Ownership

## Single chain-state owner

`ethean-node::chain::owner::ChainStateOwner` is the only component allowed to hold mutable canonical chain state. Its owned aggregate contains:

- the current canonical `ChainState`;
- fork-choice store and latest messages;
- justified and finalized checkpoints;
- canonical head and ancestry index;
- import sequence number; and
- persistence watermark for recovery.

The owner processes one `NodeCommand` at a time in a serialized event loop. Expensive pure verification may run concurrently against an immutable snapshot, but the owner rechecks the expected parent root and import sequence before committing a result. No other service receives `&mut ChainState`, a mutable fork-choice store, or a writable storage transaction.

## Chain import flow

1. `ethean-network` receives bytes and enforces peer, stream, rate, and message-size limits.
2. `ethean-network-wire` decodes a versioned bounded envelope and converts it to domain types.
3. Network publishes `NetworkEvent::BlockReceived { peer, block }`; it does not validate consensus semantics.
4. The composition root forwards the block as `NodeCommand::ImportBlock` with source metadata.
5. The node owner performs cheap duplicate, known-invalid, parent-availability, slot-window, and profile checks.
6. The node captures an immutable pre-state snapshot and invokes `ethean-crypto` and `ethean-transition`.
7. Transition returns a candidate post-state and deterministic outcome. It performs no I/O.
8. The node feeds the accepted block and operations into pure `ethean-fork-choice`, obtaining a candidate store and head decision.
9. The node compares the import sequence and parent assumptions with current ownership state. A stale result is retried or rejected; it is never blindly committed.
10. The node constructs one `StorageBatch` containing block record, post-state record, fork-choice record, canonical indexes, checkpoint metadata, and import watermark.
11. `ethean-storage` commits the batch atomically and returns a durable commit identifier.
12. Only after durable success does the owner replace its in-memory canonical aggregate.
13. The node emits `BlockImported`, `HeadChanged`, and `FinalizedChanged` events as applicable.
14. Network publication, RPC subscriptions, sync progress, validator scheduling, logs, and metrics consume those events.

If persistence fails, the candidate state is discarded and the visible canonical state remains unchanged. If event publication fails after commit, recovery reconstructs events from the persisted import watermark; it does not roll back valid chain data.

## Validator duty flow

1. The node derives a read-only duty snapshot from the owned chain state at a defined slot boundary.
2. `ethean-validator` receives `DutyNotification` containing profile identifier, head root, finalized checkpoint, slot, assignments, and an expiry bound.
3. Validator duty logic selects the required action without reading storage or chain internals.
4. For attest/aggregate duties, it builds the signing message from the supplied snapshot and calls the signer abstraction.
5. For proposal duties, validator sends `NodeCommand::PrepareProposal { slot, validator, expected_head }`.
6. The node owner serializes proposal preparation with imports, selects operations, and invokes the pure transition preview.
7. Node returns an unsigned proposal plus the exact signing root and state/head preconditions.
8. Validator signs and sends `NodeCommand::SubmitValidatorObject`.
9. Node verifies signature, duty, expiry, slashing constraints, and unchanged preconditions.
10. Node records any required slashing-protection or proposal metadata durably, then emits `PublishObject`.
11. Network encodes and gossips the object. Publication failure is reported but cannot rewrite chain state.

Validator code cannot directly gossip an object, write slashing records, or alter canonical state. This keeps local validator actions on the same validation and persistence path as remote objects.

## Network request and gossip flow

### Inbound gossip

```text
transport limits
  -> bounded wire decode
  -> topic/version consistency
  -> domain conversion
  -> node import command
  -> accept/reject/ignore receipt
  -> peer scoring response
```

Peer scoring consumes the node receipt but never influences consensus validity. Invalid encoding is rejected at the wire boundary; invalid semantics is rejected by node/core logic. The result distinguishes malformed, context-invalid, duplicate, temporarily unavailable, and accepted data.

### Outbound gossip

```text
node durable event
  -> publication command
  -> domain-to-wire conversion
  -> profile-selected topic/version
  -> transport send
  -> delivery observation
```

Only objects accepted or produced by the node enter outbound publication. Metrics may observe queueing and delivery but cannot suppress required messages except through explicit configured resource limits.

### Request/response

Sync produces bounded range/checkpoint requests. Network owns peer selection and transport retries. Responses pass through bounded decoding, request correlation, and domain conversion before sync receives them. Sync orders and batches verified candidates, then submits each batch through its node import sink. It never writes storage as a shortcut.

## Synchronization flow

1. Sync receives local status from an immutable node snapshot.
2. Sync compares peer status events and selects checkpoint or range strategy.
3. Network executes bounded requests and returns correlated responses.
4. Sync checks continuity, response bounds, roots, and cryptographic batch validity.
5. Sync sends ordered import commands through a consumer-defined `ImportSink`.
6. The composition root implements that sink with the node handle, preserving the dependency DAG.
7. Node revalidates every candidate against current canonical context and performs normal atomic persistence.
8. Node receipts advance sync progress. A receipt, not download completion, defines imported progress.

## Storage and recovery flow

At startup, the executable opens the selected backend and asks storage to verify schema version. Migrations complete before node service construction. Node recovery then:

1. reads the latest complete import watermark;
2. loads the corresponding state, fork-choice data, checkpoints, and head indexes;
3. validates root and sequence consistency in memory;
4. reconstructs `ChainStateOwner`;
5. publishes an initial immutable snapshot; and
6. only then enables network import, sync, validator duties, and RPC readiness.

Partial batches are invisible by storage contract. Missing referenced records, mismatched roots, or a future schema version stop startup with a typed error. Recovery does not guess a head from the largest slot.

## Read/query flow

RPC sends typed queries to the node handle. The node answers from an immutable snapshot or performs an explicitly bounded historical read through its persistence port. RPC never opens the database. Large historical responses use pagination and cancellation. WebSocket subscribers receive node events transformed into API view models; they do not subscribe to internal mutable structures.

## Backpressure and failure rules

- Every cross-service queue is bounded and has a documented saturation policy.
- Chain-import commands use backpressure; they are not silently dropped.
- Observational events may coalesce only when their contract permits it, such as replacing an older head snapshot with a newer one.
- Shutdown first stops ingress, then drains accepted node commands, commits storage, publishes the final watermark, and closes adapters.
- Cancellation cannot interrupt an atomic storage commit or leave the owner ahead of durable state.
- Panics in adapters terminate or restart the adapter under executable policy; they do not transfer state ownership.
