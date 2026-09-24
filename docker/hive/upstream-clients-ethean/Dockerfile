ARG devnet5_baseimage=ghcr.io/ethean-labs/ethean
ARG devnet5_tag=devnet5
ARG runtime_baseimage=ubuntu:24.04

FROM $devnet5_baseimage:$devnet5_tag AS devnet5
FROM $runtime_baseimage AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY validators.yaml /assets/lean/validators.yaml
COPY --from=devnet5 /usr/local/bin/ethean /usr/local/bin/ethean-devnet5
# The node looks for the leanMultisig prover as a sibling named exactly
# `ethean-prover` (or via ETHEAN_PROVER_BIN), so keep that name.
COPY --from=devnet5 /usr/local/bin/ethean-prover /usr/local/bin/ethean-prover
ADD ethean.sh /ethean.sh

RUN chmod +x /ethean.sh \
    && /usr/local/bin/ethean-devnet5 --version > /version-devnet5.txt \
    && printf "ethean devnet5: %s\n" \
        "$(cat /version-devnet5.txt)" > /version.txt

EXPOSE 5052 9000/udp 8080
ENTRYPOINT ["/ethean.sh"]
