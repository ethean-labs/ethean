# Rollback

## Supported

- Swap back to the previous **binary** generation when the on-disk schema major is unchanged.
- Retain checkpoint generations; resync from a trusted operator-pinned checkpoint if needed.

## Unsupported / fail closed

- Rolling back **signer journals** or completed duty watermarks (leaf reuse risk).
- Downgrading across incompatible `ethean-lc-d5-v1` schema breaks.
- Re-enabling Beacon `/eth/v1`, mock crypto accept, or TCP/WS consensus transport.

## Procedure

1. Stop new duties (`ShutdownState` drain).
2. Flush durable boundaries (storage + signer).
3. Replace binary with prior generation **only if** schema compatible.
4. Otherwise: restore prior checkpoint generation or resync from pinned trust root.
5. Confirm readiness and legacy-scan clean before returning to service.
