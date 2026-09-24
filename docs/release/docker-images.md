# Docker images

CI publishes a multi-arch image (`linux/amd64` + `linux/arm64`) to GitHub
Container Registry:

```
ghcr.io/<github-owner>/ethean
```

The image name is lowercased from `${{ github.repository_owner }}`. When the
GitHub owner is `ethean-labs` that is `ghcr.io/ethean-labs/ethean`. Day-to-day
`origin` for this clone is [ethean-labs/ethean](https://github.com/ethean-labs/ethean);
GHCR follows whoever owns the GitHub repository that runs the workflow.

The runtime image is Ubuntu 24.04 with `ethean` and `ethean-prover` in
`/usr/local/bin` (the node finds the prover as a sibling) and
`config/networks/` under `/app/config/networks`. Entrypoint is
`/usr/local/bin/ethean` (no `CMD`). Default `RUST_LOG=info`. Exposed ports:
`9000/udp` (QUIC), `5052` (Lean HTTP), `9100` (Prometheus scrape).

## Tags

| Tag | When |
| --- | --- |
| `sha-<short>` | Every build |
| `unstable` | Pushes to `master` |
| `<semver>`, `<major.minor>` | Tags `v*` |
| `latest`, `devnet5`, `latest-devnet5` | Tags `v*` |
| extra tags | `workflow_dispatch` input `extra_tags` (comma-separated) |

Workflow: [`.github/workflows/docker-image.yml`](../../.github/workflows/docker-image.yml).
Matrix: `ubuntu-latest` amd64 with `RUSTFLAGS=-Ctarget-cpu=x86-64-v3`, and
`ubuntu-24.04-arm` arm64. Each arch pushes by digest (`docker/build-push-action@v6`,
GHA cache scoped per arch); the `manifest` job runs
`docker buildx imagetools create` with tags from `docker/metadata-action@v5`
plus any dispatch extras.

## Local build

From the repository root (Docker daemon required):

```bash
docker build -t ghcr.io/ethean-labs/ethean:local .
docker run --rm ghcr.io/ethean-labs/ethean:local --version
docker run --rm --entrypoint ethean-prover ghcr.io/ethean-labs/ethean:local --version
```

Hive-oriented local image (bakes `docker/hive/ethean.sh` as entrypoint):

```bash
docker build -f docker/hive/Dockerfile -t ghcr.io/ethean-labs/ethean:local .
```

Build args: `BUILD_PROFILE` (default `release`), `FEATURES`, `LOCKED`
(default `--locked`), `RUSTFLAGS`, plus `VCS_REF` / `VCS_BRANCH` / `BUILD_DATE`
for OCI labels.

## Hive smoke (ethereum/hive sibling clone)

After copying `docker/hive/upstream-clients-ethean/` to `hive/clients/ethean/`
(already applied in the workspace clone) and with the Docker daemon up, from
the hive repository root:

```bash
./hive --sim lean \
    --client-file simulators/lean/clients/devnet5.yaml \
    --client ethean \
    --docker.output \
    --results-root ./workspace/logs
```

Devnet4:

```bash
./hive --sim lean \
    --client-file simulators/lean/clients/devnet4.yaml \
    --client ethean \
    --docker.output \
    --results-root ./workspace/logs
```

Hive pulls `ghcr.io/ethean-labs/ethean:devnet5` unless you override
`devnet5_baseimage` / `devnet5_tag` on `clients/ethean/Dockerfile`. To iterate
on a local root image first:

```bash
docker build -t ghcr.io/ethean-labs/ethean:devnet5 .
```

Registration checklist: [`docs/hive-testing/hive-quickstart-registration-2026-09-24.md`](../hive-testing/hive-quickstart-registration-2026-09-24.md).

The builder stage copies `vendor/` before `cargo chef cook`: the leanVM Windows
overlays are `[patch]` path crates, and cargo-chef does not recreate them from
the recipe (build fails with `failed to read vendor/leanvm-windows/.../Cargo.toml`).
