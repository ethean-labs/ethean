# Observability blocked by broken WSL (2026-09-20)

## Symptom

`.\scripts\run-observability.ps1` correctly throws: Docker daemon not running /
`npipe://./pipe/docker_engine`. Starting Docker Desktop is not enough on this
machine.

## Local diagnosis (Windows)

- Docker Desktop present at
  `%LOCALAPPDATA%\Programs\DockerDesktop\Docker Desktop.exe`
- CLI works; `docker info` has Client only (no Server)
- `\\.\pipe\docker_engine` never appears
- Host logs: `backend process exited` repeatedly
- `wsl -l -v` / `wsl --status` fail with access/file errors
  ("Sistem dosyaya erişemiyor")
- `com.docker.service` not registered (1060)

Conclusion: Linux engine depends on WSL2; broken WSL keeps the Docker backend
in a crash loop. Grafana/Prometheus cannot start until WSL is repaired.

## Operator fix

1. Admin PowerShell: `wsl --install` or `wsl --update`, then reboot.
2. Confirm `wsl -l -v` lists a running distro.
3. Start Docker Desktop; wait for Engine running.
4. Re-run `.\scripts\run-observability.ps1`.

Ethean `:9100/metrics` does not need Docker.
