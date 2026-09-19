# Consensus (Lean)

Protocol direction follows leanSpec / lstar (3SF-mini today), not Casper FFG epoch finality or LMD-GHOST Beacon fork choice.

## Implemented surfaces

- State transition: `ethean-transition`
- Fork choice / safe target: `ethean-fork-choice`
- Duty ticks and suppressions: `ethean-validator`
- Chain mutation: `ethean-node::ChainOwner`

## Explicit non-goals

- No BLS aggregation or Beacon committee lifecycle
- No prevote/precommit Tendermint rounds as the production path
- No `/eth/v1` duty APIs

Pins and fixtures live under `spec/`. Migration phases 05–09 in `road-to/lean-consensus-migration/`.
