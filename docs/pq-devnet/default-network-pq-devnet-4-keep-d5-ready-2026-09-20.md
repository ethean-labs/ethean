# Default network: pq-devnet-4 (keep pq-devnet-5 ready)

Date: 2026-09-20

## Why

Public **pq-devnet-5** still has no always-on joinable mesh (empty operator bootnode
list on leanroadmap). Day-to-day runs should target **pq-devnet-4**, while all D5
CLI/config/script paths stay prepared for the next operator run.

## What changed

| Surface | Before | After |
| --- | --- | --- |
| CLI default | `pq-devnet-5` | `pq-devnet-4` |
| `StartConfig` / smoke helpers | D5 | D4 (`NetworkTarget::pq_devnet_5()` kept) |
| Config files | D5 only | D4 + D5 stubs under `config/networks/` |
| Scripts | none | `scripts/run-pq-devnet-4.*` and `run-pq-devnet-5.*` |
| README Quick Start | D5-only | D4 operational + D5 ready-path section |

## Commands

```bash
# Operational (default)
ethean start --until-signal
ethean start --until-signal --network pq-devnet-4
./scripts/run-pq-devnet-4.sh

# Ready path (explicit)
ethean start --until-signal --network pq-devnet-5
./scripts/run-pq-devnet-5.sh --bootnodes '…'
```

## Files

- `crates/node/src/network_target.rs` — `PqDevnet4` / `PqDevnet5` / `Local`
- `crates/node/src/cli.rs`, `start_config.rs`, `boot_network.rs`
- `config/networks/pq-devnet-{4,5}.{bootnodes,forkdigest}`
- `scripts/run-pq-devnet-{4,5}.{sh,ps1}`
