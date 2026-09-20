# Fixed QUIC listen port (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `StartConfig::listen_port` | Default `9000` (`0` = ephemeral) |
| CLI | `--listen-port` |
| `boot_network` | QuicSwarm binds the requested port; UDP probe stays ephemeral when QUIC is on |
| Hive | `--listen-port ${HIVE_LISTEN_PORT:-9000}`, `EXPOSE 9000/udp` |

## Why

Hive and peer multiaddr discovery expect a stable `/udp/9000/quic-v1` listen.
Earlier boot used `listen_port: 0` (OS-assigned), so dialable addresses changed every run.

## Smoke

```bash
ethean start --until-signal --ephemeral --listen-port 9000
# logs should show QuicSwarm listen containing /udp/9000/quic-v1
```

## Still open

- Consume Hive `config.yaml` / validator registry files
- Upstream ethereum/hive `clients/ethean` registration
