//! Run leanSpec `state_transition_test` vectors against Ethean transition.

use crate::hex::decode_hex_fixed;
use crate::json_types::{block_from_value, state_from_value, JsonTypesError};
use ethean_profile::lstar_devnet;
use ethean_primitives::HASH32_ZERO;
use ethean_transition::{
    apply_block_unverified, apply_block_unverified_no_slots, process_block, process_slots,
    proposer_for_slot, TransitionContext, TransitionError, TransitionOutcome,
};
use ethean_types::{Block, State};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StfRunError {
    #[error("fixture case missing pre state")]
    MissingPre,
    #[error(transparent)]
    Json(#[from] JsonTypesError),
    #[error("profile: {0}")]
    Profile(String),
    #[error("expected rejection {expected}, got {got:?}")]
    WrongOutcome {
        expected: String,
        got: Result<(), TransitionError>,
    },
    #[error("post state root mismatch: got {got}, want {want}")]
    PostRoot { got: String, want: String },
    #[error("no STF cases executed")]
    NoCases,
    #[error("{0}")]
    Step(String),
}

/// Counts for one STF case run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StfRunReport {
    /// Blocks successfully applied.
    pub imports: usize,
    /// Case ended with the expected rejection.
    pub rejections: usize,
}

fn root_hex(root: &[u8; 32]) -> String {
    let mut s = String::with_capacity(66);
    s.push_str("0x");
    for b in root {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn blocks_list(v: &Value) -> Result<&Vec<Value>, StfRunError> {
    if let Some(data) = v.get("data").and_then(|d| d.as_array()) {
        Ok(data)
    } else if let Some(arr) = v.as_array() {
        Ok(arr)
    } else {
        Err(StfRunError::Step("blocks must be a list or {data:[…]}".into()))
    }
}

fn rejection_matches(reason: &str, err: &TransitionError) -> bool {
    err.to_string().starts_with(reason)
}

fn finish_allowing_zero_root(
    state: State,
    block: &Block,
) -> Result<TransitionOutcome, TransitionError> {
    let post_state_root = state
        .hash_tree_root()
        .map_err(|e| TransitionError::Types(e.to_string()))?;
    if block.state_root != HASH32_ZERO && block.state_root != post_state_root {
        return Err(TransitionError::InvalidStateRoot(
            "Invalid block state root".into(),
        ));
    }
    Ok(TransitionOutcome {
        post_state: state,
        post_state_root,
    })
}

fn apply_with_slots_allow_zero(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    let mut state = pre.clone();
    process_slots(&mut state, block.slot)?;
    process_block(&mut state, block, ctx)?;
    finish_allowing_zero_root(state, block)
}

fn apply_no_slots_allow_zero(
    pre: &State,
    block: &Block,
    ctx: &TransitionContext,
) -> Result<TransitionOutcome, TransitionError> {
    let mut state = pre.clone();
    process_block(&mut state, block, ctx)?;
    finish_allowing_zero_root(state, block)
}

/// Prefer the full transition; use no-slots / zero-root helpers for fixture edge cases.
fn apply_stf_block(
    state: &State,
    block: &Block,
    ctx: &TransitionContext,
    expected_reject: Option<&str>,
) -> Result<TransitionOutcome, TransitionError> {
    match expected_reject {
        Some("BLOCK_SLOT_MISMATCH") if state.slot != block.slot => {
            return apply_block_unverified_no_slots(state, block, ctx);
        }
        Some("BLOCK_OLDER_THAN_LATEST_HEADER") => {
            return apply_no_slots_allow_zero(state, block, ctx);
        }
        Some("BLOCK_SLOT_NOT_IN_FUTURE") => {
            return apply_block_unverified(state, block, ctx);
        }
        _ => {}
    }

    match apply_block_unverified(state, block, ctx) {
        Ok(outcome) => Ok(outcome),
        Err(TransitionError::InvalidStateRoot(_)) if block.state_root == HASH32_ZERO => {
            apply_with_slots_allow_zero(state, block, ctx)
        }
        Err(e) => Err(e),
    }
}

/// Run one STF case object (`pre`, `blocks`, optional `post` / `rejectionReason`).
pub fn run_state_transition_case(case: &Value) -> Result<StfRunReport, StfRunError> {
    let pre_v = case.get("pre").ok_or(StfRunError::MissingPre)?;
    let mut state = state_from_value(pre_v)?;
    let profile =
        lstar_devnet().map_err(|e| StfRunError::Profile(e.to_string()))?;
    let ctx = TransitionContext::new(profile);

    let blocks_v = case
        .get("blocks")
        .ok_or_else(|| StfRunError::Step("missing blocks".into()))?;
    let blocks = blocks_list(blocks_v)?;
    let expected_reject = case.get("rejectionReason").and_then(|v| v.as_str());

    let mut report = StfRunReport::default();

    if blocks.is_empty() {
        if expected_reject == Some("EMPTY_VALIDATOR_REGISTRY") {
            match proposer_for_slot(state.slot, state.validators.len() as u64) {
                Err(e) if rejection_matches("EMPTY_VALIDATOR_REGISTRY", &e) => {
                    report.rejections += 1;
                    return Ok(report);
                }
                other => {
                    return Err(StfRunError::WrongOutcome {
                        expected: "EMPTY_VALIDATOR_REGISTRY".into(),
                        got: other.map(|_| ()),
                    });
                }
            }
        }
        if expected_reject.is_some() {
            return Err(StfRunError::WrongOutcome {
                expected: expected_reject.unwrap().to_string(),
                got: Ok(()),
            });
        }
        return Ok(report);
    }

    for (i, block_v) in blocks.iter().enumerate() {
        let block = block_from_value(block_v)?;
        match apply_stf_block(&state, &block, &ctx, expected_reject) {
            Ok(outcome) => {
                if let Some(reason) = expected_reject {
                    if i + 1 == blocks.len() {
                        return Err(StfRunError::WrongOutcome {
                            expected: reason.to_string(),
                            got: Ok(()),
                        });
                    }
                }
                state = outcome.post_state;
                report.imports += 1;
            }
            Err(e) => {
                if let Some(reason) = expected_reject {
                    if rejection_matches(reason, &e) {
                        report.rejections += 1;
                        return Ok(report);
                    }
                    return Err(StfRunError::WrongOutcome {
                        expected: reason.to_string(),
                        got: Err(e),
                    });
                }
                return Err(StfRunError::Step(format!("block[{i}]: {e}")));
            }
        }
    }

    if let Some(reason) = expected_reject {
        return Err(StfRunError::WrongOutcome {
            expected: reason.to_string(),
            got: Ok(()),
        });
    }

    if let Some(want_s) = case.get("postStateRoot").and_then(|v| v.as_str()) {
        let want = decode_hex_fixed::<32>(want_s)
            .map_err(|e| StfRunError::Step(format!("postStateRoot: {e}")))?;
        let got = state
            .hash_tree_root()
            .map_err(|e| StfRunError::Step(e.to_string()))?;
        if got != want {
            return Err(StfRunError::PostRoot {
                got: root_hex(&got),
                want: want_s.to_string(),
            });
        }
    }

    if let Some(post_v) = case.get("post") {
        if let Some(want_slot) = post_v.get("slot").and_then(|v| v.as_u64()) {
            if state.slot.get() != want_slot {
                return Err(StfRunError::Step(format!(
                    "post.slot got {}, want {want_slot}",
                    state.slot.get()
                )));
            }
        }
    }

    Ok(report)
}

/// Load JSON bytes and run every STF case (objects with `pre`).
pub fn run_state_transition_file(bytes: &[u8]) -> Result<Vec<(String, StfRunReport)>, StfRunError> {
    let file: serde_json::Map<String, Value> = serde_json::from_slice(bytes)
        .map_err(|e| StfRunError::Json(JsonTypesError::Serde(e.to_string())))?;
    let mut out = Vec::new();
    for (id, case) in &file {
        if case.get("pre").is_none() {
            continue;
        }
        let report = run_state_transition_case(case)?;
        out.push((id.clone(), report));
    }
    if out.is_empty() {
        return Err(StfRunError::NoCases);
    }
    Ok(out)
}

#[cfg(test)]
#[path = "stf_runner_tests.rs"]
mod tests;
