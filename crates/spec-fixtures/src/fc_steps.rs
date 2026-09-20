//! Apply individual leanSpec fork-choice fixture steps.

use crate::fc_runner::FcRunError;
use crate::json_types::{attestation_from_value, block_from_value, signed_aggregated_from_value};
use crate::rejection::map_fork_choice_rejection;
use ethean_fork_choice::{ForkChoiceError, ForkChoiceStore};
use ethean_transition::{apply_block_unverified, TransitionContext};
use ethean_types::State;
use serde_json::Value;

/// Apply a `stepType=tick` (`interval` count or wall-clock `time` seconds).
pub fn apply_tick(
    store: &mut ForkChoiceStore,
    step: &Value,
) -> Result<(), FcRunError> {
    let target = if let Some(interval) = step.get("interval").and_then(|v| v.as_u64()) {
        interval
    } else if let Some(wall_secs) = step.get("time").and_then(|v| v.as_u64()) {
        // leanSpec / Gean: Unix seconds → interval count via genesis + ms/interval.
        let genesis_ms = store.genesis_time.saturating_mul(1000);
        let timestamp_ms = wall_secs.saturating_mul(1000);
        if timestamp_ms < genesis_ms || store.milliseconds_per_interval == 0 {
            0
        } else {
            (timestamp_ms - genesis_ms) / store.milliseconds_per_interval
        }
    } else {
        return Err(FcRunError::Step(
            "tick step missing interval/time".into(),
        ));
    };
    let has_proposal = step
        .get("hasProposal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if target < store.time {
        // Fixtures may pin time behind the clock for later checks; structural
        // runner treats a past tick as a no-op once the store has already advanced.
        return Ok(());
    }
    store
        .on_tick_with(target, has_proposal)
        .map_err(|e| FcRunError::Step(format!("on_tick({target}): {e}")))?;
    Ok(())
}

fn maybe_tick_to_slot(
    store: &mut ForkChoiceStore,
    step: &Value,
    slot: u64,
) -> Result<(), FcRunError> {
    // Absent tickToSlot defaults to true (leanSpec / Gean fixture runners).
    if step.get("tickToSlot").and_then(|v| v.as_bool()) == Some(false) {
        return Ok(());
    }
    let need = slot.saturating_mul(store.intervals_per_slot);
    if store.time < need {
        store
            .on_tick_with(need, false)
            .map_err(|e| FcRunError::Step(format!("tickToSlot({slot}): {e}")))?;
    }
    Ok(())
}

/// Advance clock to the earliest interval that admits a slot-N vote (Gean/leanSpec).
fn maybe_tick_to_admit(store: &mut ForkChoiceStore, slot: u64) -> Result<(), FcRunError> {
    let start = slot.saturating_mul(store.intervals_per_slot);
    let need = start.saturating_sub(store.gossip_disparity_intervals);
    if store.time < need {
        store
            .on_tick_with(need, false)
            .map_err(|e| FcRunError::Step(format!("admitTick({slot}): {e}")))?;
    }
    Ok(())
}

/// Apply a `stepType=block` (import or expected rejection).
pub fn apply_block_step(
    store: &mut ForkChoiceStore,
    step: &Value,
    ctx: &TransitionContext,
) -> Result<BlockStepKind, FcRunError> {
    let valid = step.get("valid").and_then(|v| v.as_bool());
    let block_v = step
        .get("block")
        .ok_or_else(|| FcRunError::Step("block step missing block".into()))?;
    let incoming = block_from_value(block_v)?;
    maybe_tick_to_slot(store, step, incoming.slot.get())?;

    if valid == Some(false) {
        let reason = step
            .get("rejectionReason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FcRunError::Step("rejected block missing rejectionReason".into()))?;
        let token = map_fork_choice_rejection(reason)
            .ok_or_else(|| FcRunError::Unmapped(reason.to_string()))?;
        let got = store.on_block(incoming, State::default());
        let expected = token.to_error();
        match &got {
            Err(e) if e == &expected => Ok(BlockStepKind::Rejected),
            other => Err(FcRunError::WrongOutcome {
                expected: reason.to_string(),
                got: other.clone(),
            }),
        }
    } else if valid == Some(true) || valid.is_none() {
        let parent = store
            .block_states
            .get(&incoming.parent_root)
            .ok_or(ForkChoiceError::UnknownParent)
            .map_err(|e| FcRunError::Step(format!("valid block parent: {e}")))?
            .clone();
        let outcome = apply_block_unverified(&parent, &incoming, ctx)
            .map_err(|e| FcRunError::Step(format!("apply_block_unverified: {e}")))?;
        store
            .on_block(incoming, outcome.post_state)
            .map_err(|e| FcRunError::Step(format!("on_block import: {e}")))?;
        Ok(BlockStepKind::Imported)
    } else {
        Err(FcRunError::Step("block step has invalid valid flag".into()))
    }
}

/// Apply a `stepType=attestation` (structural vote ingest / rejection).
pub fn apply_attestation_step(
    store: &mut ForkChoiceStore,
    step: &Value,
) -> Result<BlockStepKind, FcRunError> {
    let valid = step.get("valid").and_then(|v| v.as_bool());
    let att_v = step
        .get("attestation")
        .ok_or_else(|| FcRunError::Step("attestation step missing attestation".into()))?;
    let (validator, data) = attestation_from_value(att_v)?;
    if valid != Some(false) {
        maybe_tick_to_admit(store, data.slot.get())?;
    }

    if valid == Some(false) {
        let reason = step
            .get("rejectionReason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FcRunError::Step("rejected attestation missing rejectionReason".into())
            })?;
        let token = map_fork_choice_rejection(reason)
            .ok_or_else(|| FcRunError::Unmapped(reason.to_string()))?;
        let got = store.on_attestation_data(validator, data);
        let expected = token.to_error();
        match &got {
            Err(e) if e == &expected => Ok(BlockStepKind::Rejected),
            other => Err(FcRunError::WrongOutcome {
                expected: reason.to_string(),
                got: other.clone(),
            }),
        }
    } else if valid == Some(true) || valid.is_none() {
        store
            .on_attestation_data(validator, data)
            .map_err(|e| FcRunError::Step(format!("on_attestation_data: {e}")))?;
        Ok(BlockStepKind::Imported)
    } else {
        Err(FcRunError::Step(
            "attestation step has invalid valid flag".into(),
        ))
    }
}

/// Apply a `stepType=gossipAggregatedAttestation` (structural aggregate ingest).
pub fn apply_gossip_aggregated_step(
    store: &mut ForkChoiceStore,
    step: &Value,
) -> Result<BlockStepKind, FcRunError> {
    let valid = step.get("valid").and_then(|v| v.as_bool());
    let att_v = step
        .get("attestation")
        .ok_or_else(|| FcRunError::Step("gossipAggregatedAttestation missing attestation".into()))?;
    let (data, participants) = signed_aggregated_from_value(att_v)?;
    if valid != Some(false) {
        maybe_tick_to_admit(store, data.slot.get())?;
    }

    if valid == Some(false) {
        let reason = step
            .get("rejectionReason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FcRunError::Step("rejected gossip aggregate missing rejectionReason".into())
            })?;
        let token = map_fork_choice_rejection(reason)
            .ok_or_else(|| FcRunError::Unmapped(reason.to_string()))?;
        let got = store.on_aggregated_attestation(data, &participants);
        let expected = token.to_error();
        match &got {
            Err(e) if e == &expected => Ok(BlockStepKind::Rejected),
            other => Err(FcRunError::WrongOutcome {
                expected: reason.to_string(),
                got: other.clone(),
            }),
        }
    } else if valid == Some(true) || valid.is_none() {
        store
            .on_aggregated_attestation(data, &participants)
            .map_err(|e| FcRunError::Step(format!("on_aggregated_attestation: {e}")))?;
        Ok(BlockStepKind::Imported)
    } else {
        Err(FcRunError::Step(
            "gossipAggregatedAttestation has invalid valid flag".into(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStepKind {
    Imported,
    Rejected,
}
