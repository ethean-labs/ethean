//! Attestation structural checks and process entry.

use ethean_types::{AggregatedAttestation, AttestationData, State};

use crate::context::TransitionContext;
use crate::error::TransitionError;
use crate::operation::justify::apply_justifications;

/// Soft structural ordering checks on attestation data.
pub fn check_attestation_data_structure(data: &AttestationData) -> Result<(), TransitionError> {
    if data.source.slot.get() > data.target.slot.get() {
        return Err(TransitionError::Types(
            "attestation source slot after target".into(),
        ));
    }
    Ok(())
}

/// Count distinct `AttestationData` values (consensus cap, not SSZ list length).
pub fn distinct_attestation_data_count(attestations: &[AggregatedAttestation]) -> usize {
    let mut seen: Vec<AttestationData> = Vec::new();
    for a in attestations {
        if !seen.iter().any(|d| d == &a.data) {
            seen.push(a.data);
        }
    }
    seen.len()
}

/// Apply attestations and update justification / finalization (leanSpec `process_attestations`).
pub fn process_attestations(
    state: &mut State,
    attestations: &[AggregatedAttestation],
    ctx: &TransitionContext,
) -> Result<(), TransitionError> {
    let cap = ctx.max_attestations_data();
    let distinct = distinct_attestation_data_count(attestations);
    if distinct > cap {
        return Err(TransitionError::AttestationDataLimit(format!(
            "Block contains {distinct} distinct AttestationData entries; maximum is {cap}"
        )));
    }

    for a in attestations {
        // Soft check only; justify path filters invalid votes.
        let _ = check_attestation_data_structure(&a.data);
    }

    apply_justifications(state, attestations)
}
