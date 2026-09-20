# Start network target notes (pq-devnet-5 ready path)

Date: 2026-09-19 (superseded default: 2026-09-20)

## Current default (2026-09-20)

CLI default is **`pq-devnet-4`**. See
[default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md](default-network-pq-devnet-4-keep-d5-ready-2026-09-20.md).

## pq-devnet-5 ready path (still supported)

- `ethean start --network pq-devnet-5`
- Bootnodes from `--bootnodes`, `ETHEAN_BOOTNODES`, or `config/networks/pq-devnet-5.bootnodes`
- Fork digest from `--fork-digest`, `ETHEAN_FORK_DIGEST`, or `config/networks/pq-devnet-5.forkdigest`
- Helper: `scripts/run-pq-devnet-5.sh` / `.ps1`
- Empty list → WARN + offline local duties under the D5 label

## Historical note (2026-09-19)

Earlier builds briefly defaulted `--network` to `pq-devnet-5` before it was clear
no public D5 mesh was joinable. That default was rolled to D4; D5 code and docs
remain as the explicit ready path.

## Still required for a real D5 mesh join

Operator multiaddrs + matching leanSpec fork digest + production leanSig/leanVM gates.
