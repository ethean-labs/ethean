//! leanSpec `slot_clock`, `justifiability`, `poseidon_permutation` and `sync` suites.

use crate::driver::decode_hex_bytes;
use ethean_crypto::poseidon::{permute16, permute24};
use ethean_crypto::Fp;
use ethean_genesis::GenesisBuilder;
use ethean_primitives::{Bytes52, Slot};
use ethean_transition::is_justifiable_after;
use ethean_types::State;
use serde_json::Value;
use std::path::Path;

fn u64_of(v: &Value, key: &str) -> Result<u64, String> {
    let x = v.get(key).ok_or(format!("missing {key}"))?;
    x.as_u64()
        .or_else(|| x.as_f64().map(|f| f.max(0.0) as u64))
        .ok_or(format!("{key} is not a number"))
}

/// Intervals since genesis for a wall-clock millisecond timestamp.
pub fn intervals_since_genesis(genesis_secs: u64, now_ms: u64, ms_per_interval: u64) -> u64 {
    let genesis_ms = genesis_secs.saturating_mul(1000);
    if now_ms < genesis_ms || ms_per_interval == 0 {
        0
    } else {
        (now_ms - genesis_ms) / ms_per_interval
    }
}

/// `slot_clock` operation against the lstar timing constants.
pub fn run_slot_clock_case(case: &Value) -> Result<(), String> {
    let op = case.get("operation").ok_or("operation")?;
    let cfg = case.get("config").ok_or("config")?;
    let out = case.get("output").ok_or("output")?;
    let per_slot = u64_of(cfg, "intervalsPerSlot")?;
    let ms_per_interval = u64_of(cfg, "millisecondsPerInterval")?;
    if u64_of(cfg, "secondsPerSlot")? * 1000 != per_slot * ms_per_interval {
        return Err("config is not self-consistent".into());
    }
    let kind = op.get("kind").and_then(Value::as_str).ok_or("kind")?;
    let (got, want) = match kind {
        "from_slot" => (u64_of(op, "slot")? * per_slot, u64_of(out, "interval")?),
        "from_unix_time" => {
            let secs = u64_of(op, "unixSeconds")?;
            (
                intervals_since_genesis(u64_of(op, "genesisTime")?, secs * 1000, ms_per_interval),
                u64_of(out, "interval")?,
            )
        }
        "total_intervals" | "current_interval" | "current_slot" => {
            let total = intervals_since_genesis(
                u64_of(op, "genesisTime")?,
                u64_of(op, "currentTimeMilliseconds")?,
                ms_per_interval,
            );
            match kind {
                "total_intervals" => (total, u64_of(out, "totalIntervals")?),
                "current_interval" => (total % per_slot, u64_of(out, "interval")?),
                _ => (total / per_slot, u64_of(out, "slot")?),
            }
        }
        other => return Err(format!("unknown slot_clock kind {other}")),
    };
    if got != want {
        return Err(format!("{kind}: got {got}, want {want}"));
    }
    Ok(())
}

/// `justifiability`: `is_justifiable_after` and the slot delta.
pub fn run_justifiability_case(case: &Value) -> Result<(), String> {
    let slot = u64_of(case, "slot")?;
    let finalized = u64_of(case, "finalizedSlot")?;
    let out = case.get("output").ok_or("output")?;
    let want = out
        .get("isJustifiable")
        .and_then(Value::as_bool)
        .ok_or("isJustifiable")?;
    if u64_of(out, "delta")? != slot.saturating_sub(finalized) {
        return Err("delta mismatch".into());
    }
    let got = is_justifiable_after(Slot::new(slot), Slot::new(finalized));
    if got != want {
        return Err(format!(
            "slot {slot} after {finalized}: got {got}, want {want}"
        ));
    }
    Ok(())
}

fn fp_list(v: &Value) -> Result<Vec<Fp>, String> {
    v.as_array()
        .ok_or("state is not a list")?
        .iter()
        .map(|x| {
            let n: u32 = x
                .as_str()
                .ok_or("element is not a string")?
                .parse()
                .map_err(|_| "element is not a u32")?;
            Fp::new(n).ok_or_else(|| "element outside the field".to_string())
        })
        .collect()
}

/// `poseidon_permutation`: widths 16 and 24 over KoalaBear.
pub fn run_poseidon_case(case: &Value) -> Result<(), String> {
    let input = fp_list(case.get("inputState").ok_or("inputState")?)?;
    let want = fp_list(case.get("outputState").ok_or("outputState")?)?;
    let got: Vec<Fp> = match u64_of(case, "width")? {
        16 => {
            let mut st: [Fp; 16] = input.try_into().map_err(|_| "width 16 needs 16 elements")?;
            permute16(&mut st);
            st.to_vec()
        }
        24 => {
            let mut st: [Fp; 24] = input.try_into().map_err(|_| "width 24 needs 24 elements")?;
            permute24(&mut st);
            st.to_vec()
        }
        w => return Err(format!("unsupported width {w}")),
    };
    if got != want {
        return Err("permutation output differs".into());
    }
    Ok(())
}

fn key_pair(keys_dir: &Path, index: usize) -> Result<(Bytes52, Bytes52), String> {
    let raw = std::fs::read(keys_dir.join(format!("{index}.json"))).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let pk = |role: &str| -> Result<Bytes52, String> {
        let hex = v
            .pointer(&format!("/{role}_keypair/public_key"))
            .and_then(Value::as_str)
            .ok_or(format!("{role} key missing"))?;
        let bytes = decode_hex_bytes(hex)?;
        Bytes52::from_slice(&bytes).map_err(|e| e.to_string())
    };
    Ok((pk("attestation")?, pk("proposal")?))
}

/// `sync` `verify_checkpoint`: the served state decodes, its registry size and
/// slot match, validity follows the registry, and for a genesis anchor the
/// bytes equal Ethean's own genesis built from the fixture key set.
pub fn run_sync_case(case: &Value, keys_dir: &Path) -> Result<(), String> {
    let op = case.get("operation").ok_or("operation")?;
    let out = case.get("output").ok_or("output")?;
    if op.get("kind").and_then(Value::as_str) != Some("verify_checkpoint") {
        return Err("unknown sync operation".into());
    }
    let n = u64_of(op, "numValidators")? as usize;
    let anchor = u64_of(op, "anchorSlot")?;
    let want_valid = out.get("valid").and_then(Value::as_bool).ok_or("valid")?;
    let bytes = decode_hex_bytes(
        out.get("stateBytes")
            .and_then(Value::as_str)
            .ok_or("stateBytes")?,
    )?;
    let state = State::ssz_decode(&bytes).map_err(|e| e.to_string())?;
    if state.validators.len() != n || state.slot.get() != anchor {
        return Err(format!(
            "decoded state has {} validators at slot {}, want {n} at {anchor}",
            state.validators.len(),
            state.slot.get()
        ));
    }
    if state.ssz_encode().map_err(|e| e.to_string())? != bytes {
        return Err("state does not re-encode to stateBytes".into());
    }
    if want_valid != (n > 0) {
        return Err("checkpoint validity rule differs (non-empty registry)".into());
    }
    if n == 0 || anchor > 0 {
        // Advanced anchors are produced by applying blocks; only the genesis
        // anchor can be rebuilt from the key set alone.
        return Ok(());
    }
    let keys = (0..n)
        .map(|i| key_pair(keys_dir, i))
        .collect::<Result<Vec<_>, _>>()?;
    let built = GenesisBuilder::new(state.config.genesis_time)
        .with_validator_keys(keys)
        .build()
        .map_err(|e| e.to_string())?;
    if built.state.ssz_encode().map_err(|e| e.to_string())? != bytes {
        return Err("genesis rebuilt from the key set differs from the served state".into());
    }
    Ok(())
}
