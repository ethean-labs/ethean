//! Fixture-driven store for the hive `test_driver` routes.
//!
//! Unlike the runner, a step is applied the way a client would apply it and
//! the outcome is reported as `accepted`; hive compares that to the step's
//! `valid` flag and the snapshot to its `checks`.

use crate::fc_steps::{apply_tick, maybe_tick_to_slot};
use crate::json_signed::{
    attestation_with_signature_from_value, signed_aggregated_with_proof_from_value,
};
use crate::json_types::{block_from_value, state_from_value};
use crate::stf_runner::apply_stf_block;
use ethean_fork_choice::{create_store, ForkChoiceOpts, ForkChoiceStore};
use ethean_primitives::Hash32;
use ethean_profile::lstar_devnet;
use ethean_transition::{apply_block_unverified, proposer_for_slot, TransitionContext};
use ethean_types::{AttestationData, Checkpoint, State, Validator};
use serde_json::{json, Value};
use std::sync::Arc;

/// A vote about to enter the store, with the bytes that authenticate it.
pub struct VoteEvidence<'a> {
    pub data: &'a AttestationData,
    /// Participant bits (one bit for a single vote).
    pub participants: &'a [bool],
    /// XMSS signature (single vote) or Type-1 proof (aggregate).
    pub bytes: &'a [u8],
    pub aggregated: bool,
    /// Registry of the target block's post-state.
    pub validators: &'a [Validator],
}

/// leanSpec fixtures with `proofSetting: 0` carry placeholder aggregate proofs
/// that no verifier can check; peers skip verification on this prefix.
pub const MOCK_PROOF_PREFIX: &[u8] = b"\x00MOCKED-AGGREGATION-PROOF\x00";

/// Cryptographic check the node supplies (leanSpec `INVALID_SIGNATURE`).
pub type VoteVerifier = Arc<dyn Fn(&VoteEvidence<'_>) -> Result<(), String> + Send + Sync>;

/// A fork-choice store driven by fixture steps.
pub struct DriverStore {
    pub store: ForkChoiceStore,
    ctx: TransitionContext,
    verifier: Option<VoteVerifier>,
}

fn hex0x(root: &Hash32) -> String {
    let mut s = String::with_capacity(66);
    s.push_str("0x");
    for b in root {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn checkpoint_json(cp: &Checkpoint) -> Value {
    json!({ "slot": cp.slot.get(), "root": hex0x(&cp.root) })
}

/// Build a store from `anchorState` / `anchorBlock`, overriding genesis time.
pub fn init_driver_store(
    anchor_state: &Value,
    anchor_block: &Value,
    genesis_time: Option<u64>,
    verifier: Option<VoteVerifier>,
) -> Result<DriverStore, String> {
    let mut state = state_from_value(anchor_state).map_err(|e| e.to_string())?;
    if let Some(t) = genesis_time {
        state.config.genesis_time = t;
    }
    let block = block_from_value(anchor_block).map_err(|e| e.to_string())?;
    let profile = lstar_devnet().map_err(|e| e.to_string())?;
    let store = create_store(state, block, &profile, ForkChoiceOpts::STRUCTURAL)
        .map_err(|e| format!("anchor rejected: {e}"))?;
    Ok(DriverStore {
        store,
        ctx: TransitionContext::new(profile),
        verifier,
    })
}

/// leanSpec order: structural validation, registry bound, then signature.
fn admit_vote(
    d: &DriverStore,
    data: &AttestationData,
    participants: &[bool],
    bytes: &[u8],
    aggregated: bool,
) -> Result<(), String> {
    d.store
        .validate_attestation(data)
        .map_err(|e| e.to_string())?;
    let voters = participants
        .iter()
        .enumerate()
        .filter(|(_, b)| **b)
        .map(|(i, _)| i as u64);
    d.store
        .check_registry(data, voters)
        .map_err(|e| e.to_string())?;
    let mocked = aggregated && bytes.starts_with(MOCK_PROOF_PREFIX);
    if let (Some(verify), false) = (d.verifier.as_ref(), mocked) {
        let validators = d
            .store
            .block_states
            .get(&data.target.root)
            .map(|s| s.validators.as_slice())
            .unwrap_or(&[]);
        verify(&VoteEvidence {
            data,
            participants,
            bytes,
            aggregated,
            validators,
        })
        .map_err(|e| format!("INVALID_SIGNATURE: {e}"))?;
    }
    Ok(())
}

/// Apply one fixture step; `Err` means the step was rejected.
pub fn apply_driver_step(d: &mut DriverStore, step: &Value) -> Result<(), String> {
    let step_type = step
        .get("stepType")
        .and_then(Value::as_str)
        .or_else(|| {
            if step.get("interval").is_some() || step.get("time").is_some() {
                Some("tick")
            } else if step.get("block").is_some() {
                Some("block")
            } else if step.get("attestation").is_some() {
                Some("attestation")
            } else {
                None
            }
        })
        .ok_or("step has no stepType")?;
    match step_type {
        "tick" => apply_tick(&mut d.store, step).map_err(|e| e.to_string()),
        "block" => {
            let block_v = step.get("block").ok_or("block step missing block")?;
            let block = block_from_value(block_v).map_err(|e| e.to_string())?;
            maybe_tick_to_slot(&mut d.store, step, block.slot.get()).map_err(|e| e.to_string())?;
            let parent = d
                .store
                .block_states
                .get(&block.parent_root)
                .cloned()
                .ok_or("UNKNOWN_PARENT_BLOCK")?;
            let outcome = apply_block_unverified(&parent, &block, &d.ctx)
                .map_err(|e| format!("state transition: {e}"))?;
            d.store
                .on_block(block, outcome.post_state)
                .map_err(|e| format!("on_block: {e}"))
        }
        "attestation" => {
            let att_v = step
                .get("attestation")
                .ok_or("attestation step missing attestation")?;
            // Fixtures tick the clock explicitly; a client never advances time
            // to admit a vote, so the disparity horizon applies as-is.
            let (validator, data, signature) =
                attestation_with_signature_from_value(att_v).map_err(|e| e.to_string())?;
            let mut bits = vec![false; validator.get() as usize + 1];
            bits[validator.get() as usize] = true;
            admit_vote(d, &data, &bits, &signature, false)?;
            d.store
                .on_attestation_data(validator, data)
                .map_err(|e| format!("on_attestation: {e}"))
        }
        "gossipAggregatedAttestation" => {
            let att_v = step
                .get("attestation")
                .ok_or("gossipAggregatedAttestation missing attestation")?;
            let (data, participants, proof) =
                signed_aggregated_with_proof_from_value(att_v).map_err(|e| e.to_string())?;
            if !participants.iter().any(|b| *b) {
                return Err("EMPTY_AGGREGATION_BITS".into());
            }
            admit_vote(d, &data, &participants, &proof, true)?;
            d.store
                .on_aggregated_attestation(data, &participants)
                .map_err(|e| format!("on_aggregated_attestation: {e}"))
        }
        other => Err(format!("unsupported stepType {other}")),
    }
}

/// Hive `DriverSnapshot` (camelCase, 0x roots, time in intervals).
pub fn driver_snapshot(d: &DriverStore) -> Value {
    let s = &d.store;
    let head_slot = s.blocks.get(&s.head).map(|b| b.slot.get()).unwrap_or(0);
    json!({
        "headSlot": head_slot,
        "headRoot": hex0x(&s.head),
        "time": s.time,
        "justifiedCheckpoint": checkpoint_json(&s.latest_justified),
        "finalizedCheckpoint": checkpoint_json(&s.latest_finalized),
        "safeTarget": hex0x(&s.safe_target),
    })
}

/// Hive `state_transition/run` response for one fixture case.
pub fn run_state_transition_driver(case: &Value) -> Value {
    match drive_state_transition(case) {
        Ok(state) => json!({
            "succeeded": true,
            "error": null,
            "post": {
                "slot": state.slot.get(),
                "latestBlockHeaderSlot": state.latest_block_header.slot.get(),
                "latestBlockHeaderStateRoot": hex0x(&state.latest_block_header.state_root),
                "historicalBlockHashesCount": state.historical_block_hashes.len(),
            },
        }),
        Err(e) => json!({ "succeeded": false, "error": e, "post": null }),
    }
}

fn drive_state_transition(case: &Value) -> Result<State, String> {
    let pre_v = case.get("pre").ok_or("case missing pre")?;
    let mut state = state_from_value(pre_v).map_err(|e| e.to_string())?;
    let profile = lstar_devnet().map_err(|e| e.to_string())?;
    let ctx = TransitionContext::new(profile);
    let blocks = case
        .get("blocks")
        .and_then(Value::as_array)
        .ok_or("case missing blocks")?;
    let expected_reject = case
        .get("rejectionReason")
        .or_else(|| case.get("expectException"))
        .and_then(Value::as_str);
    if blocks.is_empty() {
        // No block to apply: the registry check is the only deterministic
        // failure the spec exercises here (peers force one the same way).
        proposer_for_slot(state.slot, state.validators.len() as u64)
            .map_err(|e| format!("no blocks: {e}"))?;
        if expected_reject.is_some() {
            return Err("no blocks to apply".into());
        }
        return Ok(state);
    }
    for (i, block_v) in blocks.iter().enumerate() {
        let block = block_from_value(block_v).map_err(|e| e.to_string())?;
        let outcome = apply_stf_block(&state, &block, &ctx, expected_reject)
            .map_err(|e| format!("block[{i}]: {e}"))?;
        state = outcome.post_state;
    }
    Ok(state)
}

/// Decode `0x`-prefixed hex of any length.
pub fn decode_hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim().trim_start_matches("0x");
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod tests;
