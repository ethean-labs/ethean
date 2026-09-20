//! Payload-pool fields inside leanSpec `storeSnapshot`.

use crate::fc_runner::FcRunError;
use crate::hex::decode_hex_fixed;
use ethean_fork_choice::ForkChoiceStore;
use ethean_primitives::Hash32;
use serde_json::Value;

fn parse_root(s: &str) -> Result<Hash32, FcRunError> {
    decode_hex_fixed::<32>(s).map_err(|e| FcRunError::Step(format!("check root: {e}")))
}

/// Compare `knownAggregatedPayloads` / `newAggregatedPayloads` participant sets.
pub(crate) fn apply_payload_pool(
    store: &ForkChoiceStore,
    snap: &Value,
    field: &str,
    known: bool,
) -> Result<(), FcRunError> {
    let Some(list) = snap.get(field).and_then(|v| v.as_array()) else {
        return Ok(());
    };
    let pool = if known {
        &store.latest_known_payloads
    } else {
        &store.latest_new_payloads
    };
    // Fixture lists must be covered; the store may retain extra historical payloads.
    for item in list {
        let root_s = item
            .get("dataRoot")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FcRunError::Step(format!("{field} entry missing dataRoot")))?;
        let root = parse_root(root_s)?;
        let entry = pool.get(&root).ok_or_else(|| {
            FcRunError::Step(format!("storeSnapshot.{field} missing dataRoot {root_s}"))
        })?;
        let want_sets = item
            .get("participantSets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| FcRunError::Step(format!("{field} missing participantSets")))?;
        let mut want: Vec<Vec<u64>> = Vec::new();
        for set in want_sets {
            let arr = set
                .as_array()
                .ok_or_else(|| FcRunError::Step(format!("{field} participant set not array")))?;
            let mut idxs = Vec::new();
            for v in arr {
                let i = v
                    .as_u64()
                    .ok_or_else(|| FcRunError::Step(format!("{field} participant not u64")))?;
                idxs.push(i);
            }
            want.push(idxs);
        }
        let got = ethean_fork_choice::normalized_participant_sets(&entry.participant_sets);
        let want = ethean_fork_choice::normalized_participant_sets(&want);
        if got != want {
            return Err(FcRunError::Step(format!(
                "storeSnapshot.{field}[{root_s}] participantSets mismatch"
            )));
        }
    }
    Ok(())
}
