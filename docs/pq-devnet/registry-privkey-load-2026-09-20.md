# Registry privkey load (2026-09-20)

## What landed

| Piece | Change |
| --- | --- |
| `SecretKeyMaterial::from_imported` | Import Hive `*.ssz` secret bytes |
| `validator_registry` | Dual-key rows + role inference (`attestation` / `proposal`) |
| `registry_keys` | Resolve `hash-sig-keys/<file>` beside `validators.yaml` |
| `LocalProposer::from_key_record` | Install proposal key (requires `leansig-backend`) |
| `ethean start` | Loads registry keys via `lean_assets` and applies best-effort |

Without `leansig-backend`, proposal privkeys are **read and logged** but not installed
(HMAC fallback refused so XMSS genesis pubkeys are not mismatched).

## Still open

- Wire attestation registry key into attester duties
- Enable `leansig-backend` in Hive image once vendor/bigint gate is green
- Upstream ethereum/hive `clients/ethean` + preparer `SUPPORTED_CLIENTS`
