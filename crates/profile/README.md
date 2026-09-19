# ethean-profile

Immutable `ChainProfile` and the pinned `lstar_devnet()` preset from Phase 00 protocol-surface / leanSpec lstar.

Also exposes:

- `ForkId` — `fork_name()` only (`fork_identifier_bytes` still unresolved)
- `ProfileLimits` / `limits_of` — registry and attestation bounds from a profile
- `profiles/pinned.toml` — data citation of lstar timing/bounds

Operational node settings (API bind, P2P listen) stay in `ethean-node`.
