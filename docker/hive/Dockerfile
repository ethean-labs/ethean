# syntax=docker/dockerfile:1.7

# Hive-oriented image for ethereum/hive simulators/lean.
# Build from the Ethean repo root:
#   docker build -f docker/hive/Dockerfile -t ethpandaops/ethean:local .

FROM rust:1.85-bookworm AS builder
WORKDIR /src
RUN apt-get update \
 && apt-get install -y --no-install-recommends clang libclang-dev pkg-config protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

COPY . .
# Default features include libp2p-quic (see bin/ethean/Cargo.toml).
RUN cargo build --release -p ethean \
 && cp /src/target/release/ethean /src/ethean

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/ethean /usr/local/bin/ethean
COPY docker/hive/ethean.sh /ethean.sh
RUN chmod +x /ethean.sh \
 && /usr/local/bin/ethean version > /version.txt || true

# Lean HTTP (:5052), metrics (:9100), QUIC/UDP (:9000 default).
EXPOSE 5052 9000/udp 9100
ENTRYPOINT ["/ethean.sh"]
