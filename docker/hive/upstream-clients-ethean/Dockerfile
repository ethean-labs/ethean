# syntax=docker/dockerfile:1.7
#
# Drop-in for ethereum/hive `clients/ethean/Dockerfile`.
# Prefers a pre-built image (see docker/hive/Dockerfile in the Ethean repo).
#
# Build args (Hive `--client-file` build_args):
#   baseimage  default ethpandaops/ethean
#   tag        default local

ARG baseimage=ethpandaops/ethean
ARG tag=local
ARG runtime_baseimage=debian:bookworm-slim

FROM ${baseimage}:${tag} AS ethean_bin
FROM ${runtime_baseimage} AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=ethean_bin /usr/local/bin/ethean /usr/local/bin/ethean
ADD ethean.sh /ethean.sh

RUN chmod +x /ethean.sh \
 && /usr/local/bin/ethean version > /version.txt || true

EXPOSE 5052 9000/udp 9100
ENTRYPOINT ["/ethean.sh"]
