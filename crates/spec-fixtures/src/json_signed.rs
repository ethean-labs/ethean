//! Fixture decoders that keep the signature / proof bytes (driver path).

use crate::json_types::{
    attestation_data, JsonAttestation, JsonSignedAggregatedAttestation, JsonTypesError,
};
use ethean_primitives::ValidatorIndex;
fn hex_bytes(s: &str) -> Result<Vec<u8>, JsonTypesError> {
    let s = s.trim().trim_start_matches("0x");
    if s.len() % 2 != 0 {
        return Err(JsonTypesError::Serde("odd hex length".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| JsonTypesError::Serde(e.to_string()))
        })
        .collect()
}

/// Signed aggregated attestation with its proof bytes (`proof.proof.data`).
pub fn signed_aggregated_with_proof_from_value(
    v: &serde_json::Value,
) -> Result<(ethean_types::AttestationData, Vec<bool>, Vec<u8>), JsonTypesError> {
    let j: JsonSignedAggregatedAttestation =
        serde_json::from_value(v.clone()).map_err(|e| JsonTypesError::Serde(e.to_string()))?;
    let proof_hex = j
        .proof
        .proof
        .as_ref()
        .and_then(|p| {
            p.get("data")
                .and_then(|d| d.as_str())
                .or_else(|| p.as_str())
        })
        .unwrap_or("0x");
    Ok((
        attestation_data(&j.data)?,
        j.proof.participants.data,
        hex_bytes(proof_hex)?,
    ))
}

/// Signed attestation with its XMSS signature bytes.
pub fn attestation_with_signature_from_value(
    v: &serde_json::Value,
) -> Result<(ValidatorIndex, ethean_types::AttestationData, Vec<u8>), JsonTypesError> {
    let j: JsonAttestation =
        serde_json::from_value(v.clone()).map_err(|e| JsonTypesError::Serde(e.to_string()))?;
    let signature = match j.signature.as_deref() {
        Some(hex) => hex_bytes(hex)?,
        None => Vec::new(),
    };
    Ok((
        ValidatorIndex::new(j.validator_index),
        attestation_data(&j.data)?,
        signature,
    ))
}
