//! Run leanSpec `fork_choice_test` steps against Ethean fork-choice.

use crate::envelope::FixtureCase;
use crate::fc_checks::apply_step_assertions;
use crate::fc_steps::{
    apply_attestation_step, apply_block_step, apply_gossip_aggregated_step, apply_tick,
    BlockStepKind,
};
use crate::json_types::{block_from_value, state_from_value, JsonTypesError};
use ethean_fork_choice::{create_store, ForkChoiceError, ForkChoiceOpts, ForkChoiceStore};
use ethean_profile::lstar_devnet;
use ethean_transition::TransitionContext;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FcRunError {
    #[error("fixture case missing anchorState / anchorBlock")]
    MissingAnchor,
    #[error(transparent)]
    Json(#[from] JsonTypesError),
    #[error("create_store: {0}")]
    Create(ForkChoiceError),
    #[error("expected rejection {expected}, got {got:?}")]
    WrongOutcome {
        expected: String,
        got: Result<(), ForkChoiceError>,
    },
    #[error("unmapped rejectionReason: {0}")]
    Unmapped(String),
    #[error("no rejection block steps executed")]
    NoSteps,
    #[error("{0}")]
    Step(String),
}

/// Counts of executed fork-choice fixture steps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FcRunReport {
    /// Successful `tick` steps.
    pub ticks: usize,
    /// Successful valid block imports.
    pub imports: usize,
    /// Successful valid attestation / vote ingests.
    pub attestations: usize,
    /// Rejection steps that matched the expected error.
    pub rejections: usize,
    /// Steps that carried `checks` and/or `storeSnapshot` and passed them.
    pub assertions: usize,
}

/// Create a store from fixture anchors and run tick / block steps in order.
pub fn run_fork_choice_case(case: &FixtureCase) -> Result<FcRunReport, FcRunError> {
    let anchor_state_v = case
        .rest
        .get("anchorState")
        .ok_or(FcRunError::MissingAnchor)?;
    let anchor_block_v = case
        .rest
        .get("anchorBlock")
        .ok_or(FcRunError::MissingAnchor)?;
    let state = state_from_value(anchor_state_v)?;
    let block = block_from_value(anchor_block_v)?;
    let profile = lstar_devnet()
        .map_err(|e| FcRunError::Create(ForkChoiceError::Types(e.to_string())))?;
    let mut store = create_store(state, block, &profile, ForkChoiceOpts::STRUCTURAL)
        .map_err(FcRunError::Create)?;
    let ctx = TransitionContext::new(profile);

    let mut report = FcRunReport::default();
    for step in &case.steps {
        let step_v = serde_json::to_value(step)
            .map_err(|e| FcRunError::Json(JsonTypesError::Serde(e.to_string())))?;
        // Prefer explicit stepType; fall back to field presence.
        let step_type = step_v
            .get("stepType")
            .and_then(|v| v.as_str())
            .or_else(|| {
                if step_v.get("interval").is_some() {
                    Some("tick")
                } else if step_v.get("block").is_some() {
                    Some("block")
                } else {
                    None
                }
            });
        match step_type {
            Some("tick") => {
                apply_tick(&mut store, &step_v)?;
                report.ticks += 1;
            }
            Some("block") => match apply_block_step(&mut store, &step_v, &ctx)? {
                BlockStepKind::Imported => report.imports += 1,
                BlockStepKind::Rejected => report.rejections += 1,
            },
            Some("attestation") => match apply_attestation_step(&mut store, &step_v)? {
                BlockStepKind::Imported => report.attestations += 1,
                BlockStepKind::Rejected => report.rejections += 1,
            },
            Some("gossipAggregatedAttestation") => {
                match apply_gossip_aggregated_step(&mut store, &step_v)? {
                    BlockStepKind::Imported => report.attestations += 1,
                    BlockStepKind::Rejected => report.rejections += 1,
                }
            }
            _ => {
                // Other step types land later.
            }
        }
        let had_assert = step_v.get("checks").is_some() || step_v.get("storeSnapshot").is_some();
        if had_assert {
            apply_step_assertions(&store, &step_v)?;
            report.assertions += 1;
        }
    }
    if report.rejections == 0
        && report.imports == 0
        && report.ticks == 0
        && report.attestations == 0
    {
        return Err(FcRunError::NoSteps);
    }
    Ok(report)
}

/// Backward-compatible alias focused on rejection fixtures.
pub fn run_fork_choice_rejections(case: &FixtureCase) -> Result<FcRunReport, FcRunError> {
    let report = run_fork_choice_case(case)?;
    if report.rejections == 0 {
        return Err(FcRunError::NoSteps);
    }
    Ok(report)
}

/// Load JSON bytes and run every case that includes anchors.
pub fn run_fork_choice_file(bytes: &[u8]) -> Result<Vec<(String, FcRunReport)>, FcRunError> {
    let file = crate::envelope::FixtureFile::from_slice(bytes)
        .map_err(|e| FcRunError::Json(JsonTypesError::Serde(e.to_string())))?;
    let mut out = Vec::new();
    for (id, case) in &file.cases {
        if case.rest.get("anchorState").is_none() {
            continue;
        }
        let report = run_fork_choice_case(case)?;
        out.push((id.clone(), report));
    }
    if out.is_empty() {
        return Err(FcRunError::NoSteps);
    }
    Ok(out)
}

/// Keep the store type available for callers.
pub type _Store = ForkChoiceStore;

#[cfg(test)]
#[path = "fc_runner_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "fc_runner_finality_tests.rs"]
mod finality_tests;

#[cfg(test)]
#[path = "fc_runner_safe_target_tests.rs"]
mod safe_target_tests;

