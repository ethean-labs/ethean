# Hive test_driver endpoints — 2026-09-24

Hive's lean simulator runs the leanSpec spec-asset suites (`fork_choice`,
`state_transition`, `verify_signatures`) by starting the client with
`HIVE_LEAN_TEST_DRIVER=1` and `HIVE_BOOTNODES=none`, then posting fixture JSON
to four routes. Ethean now serves them.

| Route | Body | Reply |
| --- | --- | --- |
| `POST /lean/v0/test_driver/fork_choice/init` | `{anchorState, anchorBlock, genesisTime}` | `204`, or `400` when the anchor pair is rejected |
| `POST /lean/v0/test_driver/fork_choice/step` | one fixture step | `200 {accepted, error, snapshot}` |
| `POST /lean/v0/test_driver/state_transition/run` | whole case (`pre`, `blocks`, …) | `200 {succeeded, error, post}` |
| `POST /lean/v0/test_driver/verify_signatures/run` | `{anchorState, signedBlock}` | `200 {succeeded, error}` |

The snapshot carries `headSlot`, `headRoot`, `time` (intervals since genesis),
`justifiedCheckpoint`, `finalizedCheckpoint` and `safeTarget`; hive compares
it to the step's `checks` itself. Without the env switch the routes answer
`404`, so a devnet node never exposes them.

## Where it lives

- `crates/rpc/src/test_driver.rs`: the `TestDriver` trait and route handling;
  `SharedApiState::install_driver` enables it.
- `crates/spec-fixtures/src/driver.rs`: a fixture-driven `ForkChoiceStore`.
  A step is applied the way a client would apply it and reported as accepted
  or rejected; the fixture's own tick steps drive the clock, nothing ticks
  implicitly before a vote. Block steps still honour `tickToSlot`.
- `crates/node/src/test_driver.rs`: the node backend, plus the production
  verifier hook (native XMSS for single votes, leanMultisig for aggregates)
  and `verify_block_proof` for the signature suite.

## Two conformance fixes found by driving every fixture

- Votes from a validator outside the target state's registry are now rejected
  (`ForkChoiceError::ValidatorNotInState`, leanSpec `VALIDATOR_NOT_IN_STATE`),
  for single votes and every participant of an aggregate.
- Vote signatures and aggregate proofs are checked before a vote enters the
  store on the driver path (`INVALID_SIGNATURE` fixtures). Fixtures generated
  with `proofSetting: 0` carry placeholder aggregate proofs that start with
  `\0MOCKED-AGGREGATION-PROOF\0`; like the peers, the driver admits those
  without verification while single-vote XMSS signatures are always checked.

## Coverage

`cargo test -p ethean-spec-fixtures` drives all 123 fork-choice cases (690
steps) and 74 state-transition cases structurally. `cargo test --release
-p ethean-node test_driver` repeats the fork-choice suite through the node
backend with the real verifier and runs the three signature fixtures.
