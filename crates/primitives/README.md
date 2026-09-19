# ethean-primitives

Stable Lean Consensus value types: `Hash32`, `Slot`, `Epoch`, `ValidatorIndex`, `Bytes52`.

No I/O, no chain config, no `BeaconBlock`. Epoch conversion from slot always takes an explicit `slots_per_epoch` argument.
