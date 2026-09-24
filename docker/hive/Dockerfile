# syntax=docker/dockerfile:1.7

# Hive-oriented local image for ethereum/hive simulators/lean.
# Build from the Ethean repo root:
#   docker build -f docker/hive/Dockerfile -t ghcr.io/ethean-labs/ethean:local .
#
# CI publishes the same binaries as ghcr.io/ethean-labs/ethean (see
# .github/workflows/docker-image.yml and the root Dockerfile); this file is the
# no-cache, single-stage-builder variant with the flexible Hive entrypoint
# (docker/hive/ethean.sh) baked in for smoke runs without prepared assets.

FROM rust:1.98.1-bookworm AS builder
ENV RUSTUP_TOOLCHAIN=1.98.1
WORKDIR /src

COPY . .
# Default features include libp2p-quic (see bin/ethean/Cargo.toml).
# ethean-prover is the leanMultisig prover process aggregators and proposers use;
# the node finds it next to its own binary.
RUN cargo build --release --locked -p ethean -p ethean-prover \
 && cp /src/target/release/ethean /src/ethean \
 && cp /src/target/release/ethean-prover /src/ethean-prover

FROM ubuntu:24.04 AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/ethean /usr/local/bin/ethean
COPY --from=builder /src/ethean-prover /usr/local/bin/ethean-prover
COPY docker/hive/ethean.sh /ethean.sh
RUN chmod +x /ethean.sh \
 && ethean --version | head -1 > /version.txt \
 && ethean-prover --version

# Lean HTTP (:5052), metrics (:9100), QUIC/UDP (:9000 default).
EXPOSE 5052 9000/udp 9100
ENTRYPOINT ["/ethean.sh"]
