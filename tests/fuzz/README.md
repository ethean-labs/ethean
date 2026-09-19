# Fuzz harnesses (Phase 13)

Targets (to be wired with `cargo-fuzz`):

- SSZ / Snappy decode
- Network ingress frames
- RPC body parsing
- Storage record decode

Seed corpora: `spec/fixtures/phase-10` and `phase-11` negative cases.
