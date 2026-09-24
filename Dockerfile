# syntax=docker/dockerfile:1.7
#
# Ethean Lean Consensus Client image (ghcr.io/ethean-labs/ethean).
#
# Build from the repository root:
#   docker build -t ghcr.io/ethean-labs/ethean:local .
#
# Build args:
#   BUILD_PROFILE  cargo profile (release | dev | any custom profile), default release
#   FEATURES       extra cargo features for `-p ethean` (empty = defaults: libp2p-quic)
#   LOCKED         `--locked` (default) or empty to allow Cargo.lock updates
#   RUSTFLAGS      e.g. -Ctarget-cpu=x86-64-v3 (CI amd64) or -Ctarget-cpu=native
#   VCS_REF / VCS_BRANCH / BUILD_DATE  OCI label inputs (set by CI)
#
# The runtime image ships both `ethean` and `ethean-prover` in /usr/local/bin;
# the node finds the prover as a sibling of its own executable (or via
# ETHEAN_PROVER_BIN). config/networks/ (bootnodes, fork digests, aggpins) is
# copied to /app/config/networks so `--network pq-devnet-4|pq-devnet-5` can
# resolve its defaults relative to WORKDIR /app.

ARG RUST_VERSION=1.98.1

FROM rust:${RUST_VERSION}-bookworm AS chef
ARG RUST_VERSION
# rust-toolchain.toml lists rustfmt/clippy; the image toolchain already matches
# the channel, so pin it here and skip the component download.
ENV RUSTUP_TOOLCHAIN=${RUST_VERSION}
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG BUILD_PROFILE=release
ARG FEATURES=""
ARG LOCKED=--locked
ARG RUSTFLAGS=""
ENV RUSTFLAGS=${RUSTFLAGS}
ENV CARGO_TERM_COLOR=never

COPY --from=planner /app/recipe.json recipe.json
# `[patch]` overlays are path crates that cargo-chef does not rebuild from the recipe.
COPY vendor vendor
RUN set -eu; \
    FEATURE_ARGS=""; \
    if [ -n "${FEATURES}" ]; then FEATURE_ARGS="--features ${FEATURES}"; fi; \
    cargo chef cook --profile "${BUILD_PROFILE}" ${LOCKED} ${FEATURE_ARGS} \
        -p ethean -p ethean-prover --recipe-path recipe.json

COPY . .
RUN set -eu; \
    FEATURE_ARGS=""; \
    if [ -n "${FEATURES}" ]; then FEATURE_ARGS="--features ${FEATURES}"; fi; \
    cargo build --profile "${BUILD_PROFILE}" ${LOCKED} ${FEATURE_ARGS} \
        -p ethean -p ethean-prover; \
    OUT="target/${BUILD_PROFILE}"; \
    if [ "${BUILD_PROFILE}" = "dev" ]; then OUT="target/debug"; fi; \
    mkdir -p /app/dist; \
    cp "${OUT}/ethean" "${OUT}/ethean-prover" /app/dist/

FROM ubuntu:24.04 AS runtime
ARG VCS_REF=unknown
ARG VCS_BRANCH=unknown
ARG BUILD_DATE=unknown

LABEL org.opencontainers.image.title="Ethean" \
      org.opencontainers.image.description="Ethean Lean Consensus Client (ethean + ethean-prover)" \
      org.opencontainers.image.source="https://github.com/ethean-labs/ethean" \
      org.opencontainers.image.url="https://github.com/ethean-labs/ethean" \
      org.opencontainers.image.documentation="https://github.com/ethean-labs/ethean/blob/master/docs/release/docker-images.md" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.ref.name="${VCS_BRANCH}" \
      org.opencontainers.image.created="${BUILD_DATE}"

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/dist/ethean /usr/local/bin/ethean
COPY --from=builder /app/dist/ethean-prover /usr/local/bin/ethean-prover
COPY config/networks /app/config/networks

RUN ethean --version && ethean-prover --version

ENV RUST_LOG=info

# QUIC/UDP p2p, Lean HTTP API, Prometheus scrape.
EXPOSE 9000/udp 5052 9100

ENTRYPOINT ["/usr/local/bin/ethean"]
