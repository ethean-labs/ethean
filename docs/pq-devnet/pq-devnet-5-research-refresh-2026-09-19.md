# pq-devnet-5 research refresh (2026-09-19)

## Verdict

**pq-devnet-5 is the right Ethean target**, but it is **not** a finished public mesh with published bootnodes. leanroadmap still shows the D5 card as **Planned** (site stamp May 2026). Engineering reality is ahead of that card: leanSpec **Type-2 block proof** merged in May 2026, and client teams ran DevNet 5 interop / scaling through at least July 2026.

There is still **no** `pq-devnet-5.md` high-level plan under [leanEthereum/pm pq-interop](https://github.com/leanEthereum/pm/tree/main/breakout-rooms/leanConsensus/pq-interop). The last official plan file is [pq-devnet-4.md](https://github.com/leanEthereum/pm/blob/main/breakout-rooms/leanConsensus/pq-interop/pq-devnet-4.md), which explicitly motivates block-level multi-message aggregation as the next step. Bootnodes for live runs stay in operator `nodes.yaml` / lean-quickstart configs, not on leanroadmap.

Local research notes (gitignored): `bazalinacaklar/pq-devnet-5.md`, `pq-devnet-5-link-index-2026-09-19.md`, `pq-devnet-5-ethean-plan-2026-09-19.md`.

## What D5 means for clients

1. **Type-2 / MultiMessageAggregate** — one block-level proof covering multiple per-message (Type-1) aggregates plus the block signature; proof must be **splittable** back into Type-1 leaves for the next proposer.
2. **Decomposability** — publish / recover intermediate per-`attestation_data` aggregates (Interop #37 working assumption: root + intermediates).
3. **Goldfish / PQ heartbeat** — listed on leanroadmap and the [HackMD proposal](https://hackmd.io/@qYrlZEprQ1iz7NjiiEmDNQ/rJ5SU8QhWx), but leanSpec PR #717 is aggregation-only and ethlambda frames Goldfish as a **later (Devnet 6)** candidate. Ethean should treat Goldfish as **spec-gated**.

## Primary sources crawled

| Source | Takeaway |
| --- | --- |
| https://leanroadmap.org/ | D4 era “active”; D5 Planned; clients include gean/Peam from D4+ |
| https://github.com/ethereum/pm (PQ Interop issues) | Weekly agendas; #37 Type-1/2 design; #40 #717 merged; #45–#49 D5 sims |
| https://github.com/leanEthereum/leanSpec/pull/717 | Merged 2026-05-20 — aggregated block proof |
| https://github.com/ReamLabs/pqdevnet-observatory + https://observatory.leanroadmap.org/ | Metrics site; Jul-15 label observed with “No data yet” on home |
| https://blog.lambdaclass.com/ethlambda-devnet-5-and-beyond/ | D5 implemented + interop; Goldfish → D6 narrative |
| [EIP-7870](https://eips.ethereum.org/EIPS/eip-7870) | Hardware/bandwidth bench target used by PQ Interop (not a D5 feature EIP) |

## Ethean plan updates

Keep default network label `pq-devnet-5`. Ordered gaps unchanged in spirit, sharpened for Type-2:

1. Operator bootnodes + fork digest (files already exist empty under `config/networks/`).
2. leanSig production verify.
3. leanVM IPC: Type-1 merge, Type-2 prove/verify/split.
4. Status handshake completion + BlocksByRoot.
5. D5 block body decode (single Type-2 proof).
6. Hive / leanSpec fixture consumer.
7. Goldfish only when the run’s leanSpec fork requires it.

See also: [working-client-pq-devnet-5-plan-2026-09-19.md](working-client-pq-devnet-5-plan-2026-09-19.md).
