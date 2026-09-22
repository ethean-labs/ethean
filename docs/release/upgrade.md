# Upgrade

1. Build release artifacts with `tools/release/repro-build.sh`.
2. Record `artifacts/phase-13/repro/build-meta.*` and SBOM lock fingerprint.
3. Stage the new binary beside the prior generation; keep prior data directory intact.
4. Run schema open (`ethean-lc-d5-v1`); refuse Panro/JSON dirs.
5. Start node with readiness false until storage/crypto/signer/network/prover gates pass.
6. Homogeneous smoke: head/finalized converge; scrape `ethean_` metrics.
7. Only then remove the prior binary generation.

If schema version is unsupported by the old binary, do **not** downgrade data — follow [rollback.md](./rollback.md).
