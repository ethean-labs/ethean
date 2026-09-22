# Contributing to Ethean

Thanks for helping with **Ethean Lean Consensus Client** — a Rust consensus-only
implementation of Ethereum Lean Consensus (historically “Beam Chain”).

Ethean tracks [leanroadmap.org](https://leanroadmap.org/) research tracks and the
pq-devnet sequence (leanSpec / leanSig / leanVM / leanMultisig). It is **not** an
execution client and not a Beacon-chain compatibility layer.

## Before you start

1. Read the root [README.md](./README.md) Quick Start and Monitoring sections.
2. Skim [docs/README.md](./docs/README.md) for design notes that touch your area.
3. Prefer Lean / pq-devnet behavior over legacy Eth2 shortcuts.
4. Peer clients ([Ream](https://github.com/ReamLabs/ream),
   [Zeam](https://github.com/blockblaz/zeam),
   [ethlambda](https://github.com/lambdaclass/ethlambda), and others listed in
   [docs/peer-reference-clients.md](./docs/peer-reference-clients.md)) are
   **interop references**, not templates. Do not copy their crate layout or house style.

## Prerequisites

- Rust toolchain from [`rust-toolchain.toml`](./rust-toolchain.toml) (via rustup)
- Git
- Optional: Docker Engine + Compose — required for Grafana / Prometheus
  (`ethean start … --metrics`)

## Setup

```bash
git clone <your-fork-or-this-repo>
cd Ethean
cargo build -p ethean --release
cargo test
cargo clippy --workspace --all-targets
cargo fmt --check
```

After `cargo build -p ethean`, the `ethean` command is available via a PATH shim
under `~/.cargo/bin` (see [docs/ethean-path-command-after-build-2026-09-20.md](./docs/pq-devnet/ethean-path-command-after-build-2026-09-20.md)).

### Local run (operational default)

```bash
# Offline smoke under the pq-devnet-4 label (no public mesh without bootnodes)
ethean start --until-signal --network pq-devnet-4

# Scrape HTTP is on by default at http://127.0.0.1:9100/metrics
# Grafana :3000 + Prometheus :9090 need Docker:
ethean start --until-signal --network pq-devnet-4 --metrics
```

`pq-devnet-5` remains a ready path (`--network pq-devnet-5` +
`config/networks/pq-devnet-5.*`). Join a live operator mesh only with that run’s
`nodes.yaml` / multiaddrs pasted into the matching bootnodes file.

## Code guidelines

### Language

Everything in the git tree is **English**: identifiers, comments, rustdoc, CLI,
logs, tests, commit messages, and docs. Chat may follow the author’s language;
files may not. See [docs/english.md](./docs/english.md).

### File size

Authored source files stay at **300 lines or fewer**. Split by responsibility and
re-export from `mod.rs` when a file would grow past that. See
[docs/source-file-size-limit.md](./docs/source-file-size-limit.md).

### Style

- Boring, consistent Rust: small functions, explicit errors, names that match
  existing modules.
- Comments only when intent is non-obvious.
- No AI / “generated” banners, filler changelog comments, or assistant voice in
  source.

### Security and secrets

- Never commit secrets, `.env`, or private keys.
- Do not commit local research extracts under `bazalinacaklar/` (gitignored) or
  peer client clones.
- Do not commit `.cursor/` rules.

## Git workflow

### Branches and PRs

1. Branch from `master` (or the repo’s default branch).
2. Keep PRs focused: one logical change per PR when practical.
3. Ensure `cargo test`, `cargo clippy`, and `cargo fmt` are clean before asking
   for review.
4. Write an English PR title and a short body: **why**, not a file dump.
5. Do **not** advertise Cursor / Copilot / ChatGPT authorship in the PR.

### Commits

- Messages are English, 1–2 sentences, explaining **why**.
- Prefer one coherent change per commit (often one source/docs file plus its
  `mod.rs` wiring).
- Never push unless maintainers ask you to, or your fork workflow requires it.
- **No AI git attribution**: no `Co-authored-by: Cursor`, no
  `cursoragent@cursor.com`, no `Made-with: Cursor`. Commits must use your normal
  `user.name` / `user.email`. See
  [docs/no-ai-git-attribution-2026-09-19.md](./docs/process/no-ai-git-attribution-2026-09-19.md).
- Enable the repo hook when developing locally:

```bash
git config core.hooksPath .githooks
```

### Docs with each material change

After a finished development update, add a short English note under `docs/` with
a unique name and link it from [docs/README.md](./docs/README.md) / the root
README when operators need it. See [docs/commit-after-each-file.md](./docs/commit-after-each-file.md)
and [docs/versioning.md](./docs/versioning.md) for version bumps.

## What to work on

High-value areas (check recent `docs/*` notes and open issues first):

- Lean P2P (QUIC, gossip, Status / blocks-by-root / blocks-by-range)
- Fork choice / duties aligned with the current leanSpec pin
- leanSig / leanVM gates (fail-closed until backends are real)
- Observability (`ethean_` metrics, `deploy/observability`)
- Local private mesh helpers and operator bootnode plumbing

When stuck on a protocol detail, compare 1–2 peer clients, then implement in
**Ethean’s** modules. leanSpec / current pq-devnet pin wins over peer majority.

## Code of conduct

This project follows the [Contributor Covenant](./CODE_OF_CONDUCT.md)
(v2.1). Be respectful in issues and PRs. Assume good faith. Disagreements are
about the protocol and the code, not the person. Report enforcement concerns to
the contact listed in that file.

## License

By contributing, you agree that your contributions are licensed under the same
terms as this repository (see [LICENCE](./LICENCE) / license files in-tree).

## Questions

- Design notes: [docs/README.md](./docs/README.md)
- Deployment: [docs/deployment.md](./docs/deployment.md)
- Lean roadmap: https://leanroadmap.org/
