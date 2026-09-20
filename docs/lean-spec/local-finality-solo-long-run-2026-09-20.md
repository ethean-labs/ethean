# Local finality for solo long-run (2026-09-20)

## Problem

`ethean start --until-signal --network pq-devnet-5` sat for hours with
`head_slot=0` / `finalized_slot=0` because:

1. Genesis time was `1_700_000_000` → wall slot in the millions; first proposal
   tried to `process_slots` across that gap (appears hung).
2. `head_root` stayed zero → parent mismatch on proposals.
3. Published blocks were never self-applied → solo head never moved.
4. No public bootnodes (expected) and no aggregator/local-finality path.

## Fix (wired)

- Recent genesis + `--validators N` (default **4**)
- Seal genesis `head_root` before duties
- `--local-finality` **default on** (`--no-local-finality` to disable): inject
  full-registry attestation, self-apply, promote justified/finalized (1-slot lag)
- Aggregator **default on** (`--no-aggregator` to disable)
- Scripts: `scripts/run-local-finality.*`

Local checkpoint promotion is a **solo smoke** shortcut so Grafana panels climb
without a public mesh. Mixed-client interop still needs lean-quickstart + peers.

## Run

```powershell
ethean start --until-signal --network pq-devnet-4
# or with Grafana/Prometheus Docker stack:
.\scripts\run-local-finality.ps1 -MetricsStack
```

Watch http://127.0.0.1:9100/metrics for `ethean_head_slot` /
`ethean_justified_slot` / `ethean_finalized_slot` climbing.
