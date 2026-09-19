# Release gates

Security and performance gates for Phase 13 promotion. Authority:

- [SECURITY_GATES.md](../../road-to/lean-consensus-migration/04-risks/SECURITY_GATES.md)
- [PERFORMANCE_BUDGETS.md](../../road-to/lean-consensus-migration/04-risks/PERFORMANCE_BUDGETS.md)
- [DELETION_REGISTER.md](../../road-to/lean-consensus-migration/05-retirement/DELETION_REGISTER.md)

## Checklist (must archive evidence under `artifacts/phase-13/`)

1. Offline vectors green for types/crypto/transition/fork-choice
2. Single-node no-signing smoke
3. Isolated signing (test-hmac / fail-closed production)
4. Homogeneous multi-node (when network QUIC wired)
5. Mixed-client multi-node
6. Adversarial staging (chaos)
7. Targeted devnet promotion

## Non-waivable

- G1/G2 crypto isolation and XMSS leaf safety
- Checkpoint trust ≠ structural validity
- No Beacon `/eth/v1` production routes
- Signer journal never rewound by release rollback
