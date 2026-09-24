# Hive and lean-quickstart registration (2026-09-24)

Packaging-side registration of Ethean as a Lean client. No version bump.
Workspace clones stay uncommitted so they can be opened as PRs.

GHCR image name is `ghcr.io/<github-owner>/ethean` (lowercased). Docs and
Dockerfiles use `ethean-labs/ethean`. Day-to-day git `origin` for this clone is
`https://github.com/ethean-labs/ethean.git`; the GHCR namespace follows the
GitHub owner of the repository that runs `.github/workflows/docker-image.yml`.

## Ethean (this repo)

| Path | Change |
| --- | --- |
| `Dockerfile` | Multi-stage `rust:1.98.1-bookworm` + cargo-chef; runtime `ubuntu:24.04`; `ethean` + `ethean-prover` in `/usr/local/bin` |
| `.dockerignore` | Keep `config/`, `Cargo.lock`, `rust-toolchain.toml`, `LICENCE`; exclude `target/`, `.git/`, `dist/`, tmp data |
| `.github/workflows/docker-image.yml` | amd64 + arm64 push-by-digest; tags `sha-*`, `unstable`, semver, `latest` / `devnet5` / `latest-devnet5` |
| `.github/workflows/release-binaries.yml` | `REPO_URL` → `ethean-labs/ethean` |
| `Cargo.toml` | `repository` → `https://github.com/ethean-labs/ethean` |
| `tools/release/render-binaries-notes.sh` | default `REPO_URL` |
| `docker/hive/Dockerfile` | rust 1.98.1-bookworm; `ethean --version \| head -1 > /version.txt` |
| `docker/hive/upstream-clients-ethean/*` | Drop-in for `hive/clients/ethean/` (ream structure) |
| `docs/release/docker-images.md` | Tags, workflow, local / Hive commands |
| `CHANGELOG.md` | Unreleased packaging bullets |

## hive (sibling `./hive`, uncommitted)

Copy of `ethean/docker/hive/upstream-clients-ethean/` plus simulator lists:

| Path | Change |
| --- | --- |
| `clients/ethean/Dockerfile` | Wrapper of `ghcr.io/ethean-labs/ethean:devnet5`; copies `ethean-prover` as sibling |
| `clients/ethean/Dockerfile.git` | Source build `ethean-labs/ethean` |
| `clients/ethean/ethean.sh` | Ream-style HIVE_* mapping; multiaddr bootnodes including `none` |
| `clients/ethean/hive.yaml` | `roles: [lean]` |
| `clients/ethean/validators.yaml` | `ethean_0` … `ethean_15` |
| `simulators/lean/clients/devnet4.yaml` | `- client: ethean` / `nametag: devnet4` |
| `simulators/lean/clients/devnet5.yaml` | `- client: ethean` / `nametag: devnet5` |
| `simulators/lean/config/lean-devnets.txt` | `ethean=devnet4,devnet5` |
| `simulators/lean/src/utils/util.rs` | `lean_client_kind` includes `ethean`; **not** in `client_uses_enr_bootnodes` |
| `simulators/lean/helper/prepare_lean_client_assets.py` | `SUPPORTED_CLIENTS` + ream writer branch |

Hive smoke (from hive repo root, Docker daemon required):

```bash
./hive --sim lean \
    --client-file simulators/lean/clients/devnet5.yaml \
    --client ethean \
    --docker.output \
    --results-root ./workspace/logs
```

## lean-quickstart (sibling `./lean-quickstart`, branches `main` and `devnet5`, uncommitted)

| Path | Change |
| --- | --- |
| `client-cmds/ethean-cmd.sh` | Modelled on `ream-cmd.sh`; image `ghcr.io/ethean-labs/ethean:devnet5` |
| `local-devnet/genesis/validator-config.yaml` | `ethean_0` with unique ports |
| `README.md` | Integrates line + Clients supported list |
| `docs/adding-a-new-client.md` | Client list mention where present |

Ansible roles (`ansible/roles/ethean/`) are **not** in this packaging pass
(touch points 3–5 of the adding-a-new-client guide). Local `spin-node.sh`
discovers `client-cmds/ethean-cmd.sh` from the node name prefix.

## leanMetrics (sibling `./leanMetrics/dashboards`)

Job regexes and the interop `name=~` cadvisor expression already include
`ethean`. `metrics.md` already has an Ethean column; left unchanged.

## Open questions

- Day-to-day `origin` is `ethean-labs/ethean`. Confirm `ghcr.io/ethean-labs/ethean:devnet5`
  exists after CI publishes from that repository.
- Hive `Dockerfile.git` clones `https://github.com/ethean-labs/ethean`; override
  ARG `github` at build time if a fork build is needed.
