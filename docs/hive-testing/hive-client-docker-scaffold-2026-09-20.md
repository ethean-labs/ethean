# Hive client scaffold for Ethean (2026-09-20)

## What landed

Under `docker/hive/`:

| File | Role |
| --- | --- |
| `Dockerfile` | Multi-stage `cargo build -p ethean` → slim runtime |
| `ethean.sh` | Map `HIVE_*` Lean simulator env → `ethean start` |
| `client-devnet5.yaml` | Snippet to append in ethereum/hive |
| `README.md` | Build + honesty about gaps |

Image entrypoint uses `--until-signal --ephemeral` and network labels
`pq-devnet-5` / `pq-devnet-4` / `local`.

## Recipe

```bash
docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .
```

## Still open

| Gap | Notes |
| --- | --- |
| Load registry privkeys for duties | Attester wiring landed; proposal install needs `leansig-backend` |
| Upstream `clients/ethean` PR in ethereum/hive | Drop-in ready: [`docker/hive/upstream-clients-ethean/`](../../docker/hive/upstream-clients-ethean/) |
| A2/A3 / leanVM / bigint | External production gates |

Lean HTTP `:5052`, fixed QUIC `:9000`, Hive `config.yaml` / `validators.yaml`
consume, and ACC → profile/subnets landed — see
[`lean-http-api-5052-2026-09-20.md`](../networking/lean-http-api-5052-2026-09-20.md),
[`quic-fixed-listen-port-2026-09-20.md`](../networking/quic-fixed-listen-port-2026-09-20.md),
[`hive-lean-config-consume-2026-09-20.md`](hive-lean-config-consume-2026-09-20.md),
[`attestation-committee-count-profile-subnets-2026-09-20.md`](../networking/attestation-committee-count-profile-subnets-2026-09-20.md), and
[`hive-upstream-clients-ethean-dropin-2026-09-20.md`](hive-upstream-clients-ethean-dropin-2026-09-20.md).
