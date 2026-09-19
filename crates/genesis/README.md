# ethean-genesis

Deterministic genesis construction/loading and an injectable Lean slot clock for Ethean.

## Authority

- Timing comes from `ethean-profile::ChainProfile` (lstar: 4s slots, 5 intervals).
- `GenesisConfig.genesis_time` is Unix **seconds** (leanSpec `Uint64`); the clock converts to milliseconds with checked arithmetic.
- Network startup must use `GenesisBuilder` or `load_genesis_ssz` — not `State::default()`.

## Modules

| Module | Role |
| --- | --- |
| `builder` | Build slot-0 `State` from validator key pairs |
| `loader` | Load/verify genesis SSZ bytes |
| `clock` | `SlotClock`, `TimeSource`, `FakeTime`, `SystemTimeSource` |
| `error` | `GenesisError` / `ClockError` |

`SystemTimeSource` is for node ops only; consensus tests should inject `FakeTime`.
