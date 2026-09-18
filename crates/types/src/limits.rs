//! Pinned Lean container bounds (lstar / protocol-surface).

/// Maximum validators in the registry (`VALIDATOR_REGISTRY_LIMIT`).
pub const VALIDATOR_REGISTRY_LIMIT: usize = 4096;

/// Maximum historical block roots (`HISTORICAL_ROOTS_LIMIT`).
pub const HISTORICAL_ROOTS_LIMIT: usize = 262_144;

/// Maximum distinct attestation data entries per block (`MAX_ATTESTATIONS_DATA`).
pub const MAX_ATTESTATIONS_DATA: usize = 8;

/// leanSpec `AggregatedAttestations` SSZ list limit (registry-sized).
pub const AGGREGATED_ATTESTATIONS_LIMIT: usize = VALIDATOR_REGISTRY_LIMIT;

/// XMSS public key width (`Bytes52`).
pub const XMSS_PUBLIC_KEY_BYTES: usize = 52;

/// XMSS signature width (PROD_CONFIG).
pub const XMSS_SIGNATURE_BYTES: usize = 2536;

/// Maximum aggregate proof payload (`ByteList512KiB`).
pub const BYTE_LIST_512_KIB: usize = 512 * 1024;

/// JustificationValidators bitlist limit: roots × validators.
pub const JUSTIFICATION_VALIDATORS_LIMIT: usize =
    HISTORICAL_ROOTS_LIMIT * VALIDATOR_REGISTRY_LIMIT;
