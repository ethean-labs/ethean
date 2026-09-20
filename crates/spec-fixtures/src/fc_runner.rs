//! Run leanSpec `fork_choice_test` rejection steps against Ethean fork-choice.

use crate::envelope::FixtureCase;
use crate::json_types::{block_from_value, state_from_value, JsonTypesError};
use crate::rejection::map_fork_choice_rejection;
use ethean_fork_choice::{create_store, ForkChoiceError, ForkChoiceOpts, ForkChoiceStore};
use ethean_profile::lstar_devnet;
use ethean_types::State;
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
}

/// Outcome of running mapped rejection steps in one fixture case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FcRejectionReport {
    /// How many rejected block steps were executed and matched.
    pub matched: usize,
}

/// Create a store from fixture anchors and assert each rejected block step.
pub fn run_fork_choice_rejections(case: &FixtureCase) -> Result<FcRejectionReport, FcRunError> {
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
    let profile = lstar_devnet().map_err(|e| {
        FcRunError::Create(ForkChoiceError::Types(e.to_string()))
    })?;
    let mut store = create_store(state, block, &profile, ForkChoiceOpts::STRUCTURAL)
        .map_err(FcRunError::Create)?;

    let mut matched = 0;
    for step in &case.steps {
        if step.valid != Some(false) {
            continue;
        }
        let Some(reason) = step.rejection_reason.as_deref() else {
            continue;
        };
        let Some(token) = map_fork_choice_rejection(reason) else {
            return Err(FcRunError::Unmapped(reason.to_string()));
        };
        let Some(block_v) = step.rest.get("block") else {
            continue;
        };
        let incoming = block_from_value(block_v)?;
        // Rejection paths fail before post-state is consumed; dummy is enough.
        let dummy = State::default();
        let got = store.on_block(incoming, dummy);
        let expected_err = token.to_error();
        match &got {
            Err(e) if e == &expected_err => {
                matched += 1;
            }
            other => {
                return Err(FcRunError::WrongOutcome {
                    expected: reason.to_string(),
                    got: other.clone(),
                });
            }
        }
    }
    if matched == 0 {
        return Err(FcRunError::NoSteps);
    }
    Ok(FcRejectionReport { matched })
}

/// Convenience: load JSON bytes and run every case that has rejection steps.
pub fn run_fork_choice_file(bytes: &[u8]) -> Result<Vec<(String, FcRejectionReport)>, FcRunError> {
    let file = crate::envelope::FixtureFile::from_slice(bytes)
        .map_err(|e| FcRunError::Json(JsonTypesError::Serde(e.to_string())))?;
    let mut out = Vec::new();
    for (id, case) in &file.cases {
        if case
            .steps
            .iter()
            .any(|s| s.valid == Some(false) && s.rejection_reason.is_some())
        {
            // Skip cases that only have rejectionReason without block payload
            // (committed structural samples).
            if case.rest.get("anchorState").is_none() {
                continue;
            }
            let report = run_fork_choice_rejections(case)?;
            out.push((id.clone(), report));
        }
    }
    if out.is_empty() {
        return Err(FcRunError::NoSteps);
    }
    Ok(out)
}

/// Keep the store type name referenced for callers that want to inspect later.
pub type _Store = ForkChoiceStore;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::{discover_json_fixtures, fixtures_root_from_env};
    use std::path::PathBuf;

    fn beyond_horizon_path() -> Option<PathBuf> {
        if fixtures_root_from_env().is_none() {
            // Default local cache from fetch-leanspec-fixtures.ps1
            let mut def = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            def.pop();
            def.pop();
            def.push(".cache");
            def.push("leanspec-fixtures");
            def.push("extracted");
            if def.is_dir() {
                std::env::set_var(crate::FIXTURES_ENV, &def);
            }
        }
        let root = fixtures_root_from_env()?;
        let mut p = root.clone();
        p.push("fixtures");
        p.push("consensus");
        p.push("fork_choice");
        p.push("lstar");
        p.push("fork_choice");
        p.push("test_block_future_horizon");
        p.push("test_block_beyond_future_horizon_rejected.json");
        if p.is_file() {
            return Some(p);
        }
        let mut alt = root;
        alt.push("consensus");
        alt.push("fork_choice");
        alt.push("lstar");
        alt.push("fork_choice");
        alt.push("test_block_future_horizon");
        alt.push("test_block_beyond_future_horizon_rejected.json");
        alt.is_file().then_some(alt)
    }

    #[test]
    fn runs_beyond_future_horizon_when_cache_present() {
        let Some(path) = beyond_horizon_path() else {
            eprintln!("skip: set ETHEAN_LEANSPEC_FIXTURES after fetch-leanspec-fixtures.ps1");
            return;
        };
        let bytes = std::fs::read(&path).expect("read fixture");
        let reports = run_fork_choice_file(&bytes).expect("run fixture");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].1.matched, 1);
    }

    #[test]
    fn discovers_future_horizon_under_cache() {
        let Some(root) = fixtures_root_from_env() else {
            return;
        };
        let files = discover_json_fixtures(&root).unwrap();
        assert!(files.iter().any(|p| p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains("beyond_future_horizon"))));
    }
}
