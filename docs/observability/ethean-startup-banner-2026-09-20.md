# Startup ASCII banner and start snapshot (2026-09-20)

`ethean start` (and `ethean version`) print an Ethean identity block before the
normal log dump:

1. ASCII logo + product name + slogan + version
2. After the client loads: a green/cyan **start snapshot** (network kind,
   durable/ephemeral, fork, slot time, validators, roles, bootnodes, genesis
   time, head/justified/finalized/wall slots, metrics, verbosity)

Skip with `--no-banner`.

Code: `bin/ethean/src/banner_art.rs`, `bin/ethean/src/banner.rs`.
