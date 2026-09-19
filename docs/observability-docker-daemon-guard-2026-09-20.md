# Observability script: Docker daemon guard (2026-09-20)

## Problem

On Windows, `.\scripts\run-observability.ps1` could print:

`failed to connect to the docker API at npipe://./pipe/docker_engine`
(`Sistem belirtilen dosyayı bulamıyor` / file not found)

then continue into `Waiting for Prometheus...` and sit until the ready loop
timed out. Root cause: Docker CLI present, **Docker Desktop / Engine not
running** (named pipe missing). PowerShell does not treat native non-zero
exits as terminating errors by default, so `docker compose up -d` failure did
not stop the script.

## Change

`scripts/run-observability.ps1` now:

1. Checks `docker` is on PATH.
2. Runs `docker info` and exits with a clear message if the daemon is down.
3. Checks `$LASTEXITCODE` after `docker compose up -d`.

`deploy/observability/README.md` documents the Windows npipe failure and the
fix (start Docker Desktop, wait for Engine running, re-run).

## Operator action

Start Docker Desktop before the observability stack. Metrics on `:9100` from
the ethean binary do not require Docker; Grafana/Prometheus do.
