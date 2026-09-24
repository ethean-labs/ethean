//! Hive `test_driver` backend: fixture fork choice, state transition and
//! signature verification through the production code paths.

use std::sync::Mutex;

use ethean_crypto::{AggregateVerifier, CryptoBackend, ProductionBackend, PublicKey, Signature};
use ethean_multisig::LeanMultisigVerifier;
use ethean_rpc::TestDriver;
use ethean_spec_fixtures::{
    apply_driver_step, block_from_value, decode_hex_bytes, driver_snapshot, init_driver_store,
    run_state_transition_driver, state_from_value, DriverStore, VoteEvidence, VoteVerifier,
};
use ethean_transition::verify_block_proof;
use ethean_types::{MultiMessageAggregate, SignedBlock, Validator};
use serde_json::{json, Value};

/// Env switch the hive wrapper exports for spec-asset containers.
pub const DRIVER_ENV: &str = "HIVE_LEAN_TEST_DRIVER";

/// True when `HIVE_LEAN_TEST_DRIVER=1`.
pub fn enabled_by_env() -> bool {
    std::env::var(DRIVER_ENV).is_ok_and(|v| v == "1")
}

/// Driver state: one replaceable fork-choice store.
#[derive(Default)]
pub struct NodeTestDriver {
    store: Mutex<Option<DriverStore>>,
}

fn parse(body: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(body).map_err(|e| format!("invalid JSON body: {e}"))
}

impl TestDriver for NodeTestDriver {
    fn fork_choice_init(&self, body: &[u8]) -> Result<(), String> {
        let v = parse(body)?;
        let anchor_state = v.get("anchorState").ok_or("missing anchorState")?;
        let anchor_block = v.get("anchorBlock").ok_or("missing anchorBlock")?;
        let genesis = v.get("genesisTime").and_then(Value::as_u64);
        let store = init_driver_store(anchor_state, anchor_block, genesis, Some(vote_verifier()))?;
        *self.store.lock().map_err(|_| "driver lock poisoned")? = Some(store);
        Ok(())
    }

    fn fork_choice_step(&self, body: &[u8]) -> Result<String, String> {
        let step = parse(body)?;
        let mut guard = self.store.lock().map_err(|_| "driver lock poisoned")?;
        let store = guard.as_mut().ok_or("fork choice not initialised")?;
        let outcome = apply_driver_step(store, &step);
        let snapshot = driver_snapshot(store);
        let (accepted, error) = match outcome {
            Ok(()) => (true, Value::Null),
            Err(e) => (false, Value::String(e)),
        };
        Ok(json!({ "accepted": accepted, "error": error, "snapshot": snapshot }).to_string())
    }

    fn state_transition(&self, body: &[u8]) -> Result<String, String> {
        let case = parse(body)?;
        Ok(run_state_transition_driver(&case).to_string())
    }

    fn verify_signatures(&self, body: &[u8]) -> Result<String, String> {
        let case = parse(body)?;
        Ok(verify_signatures_case(&case).to_string())
    }
}

fn attestation_keys(validators: &[Validator], bits: &[bool]) -> Result<Vec<PublicKey>, String> {
    bits.iter()
        .enumerate()
        .filter(|(_, b)| **b)
        .map(|(i, _)| {
            let v = validators.get(i).ok_or("validator not in state")?;
            PublicKey::try_from_slice(v.attestation_public_key.as_bytes())
                .map_err(|e| e.to_string())
        })
        .collect()
}

/// Production XMSS / leanMultisig checks for fixture votes.
pub fn vote_verifier() -> VoteVerifier {
    std::sync::Arc::new(|e: &VoteEvidence<'_>| {
        let keys = attestation_keys(e.validators, e.participants)?;
        let root = e.data.hash_tree_root();
        if e.aggregated {
            LeanMultisigVerifier
                .verify_single(e.bytes, &keys, &root, e.data.slot.get())
                .map_err(|err| err.to_string())
        } else {
            let key = keys.first().ok_or("no voter")?;
            let signature = Signature::try_from_slice(e.bytes).map_err(|err| err.to_string())?;
            let slot = u32::try_from(e.data.slot.get()).map_err(|_| "slot beyond XMSS lifetime")?;
            let valid = ProductionBackend
                .verify(key, slot, &root, &signature)
                .map_err(|err| err.to_string())?;
            if valid {
                Ok(())
            } else {
                Err("XMSS signature does not verify".into())
            }
        }
    })
}

/// `{succeeded, error}` for a `verify_signatures` fixture case.
pub fn verify_signatures_case(case: &Value) -> Value {
    match verify_case(case) {
        Ok(()) => json!({ "succeeded": true, "error": null }),
        Err(e) => json!({ "succeeded": false, "error": e }),
    }
}

fn verify_case(case: &Value) -> Result<(), String> {
    let state = state_from_value(case.get("anchorState").ok_or("missing anchorState")?)
        .map_err(|e| e.to_string())?;
    let signed_v = case.get("signedBlock").ok_or("missing signedBlock")?;
    let block_v = signed_v
        .get("block")
        .or_else(|| signed_v.get("message"))
        .ok_or("signedBlock missing block")?;
    let block = block_from_value(block_v).map_err(|e| e.to_string())?;
    let proof_hex = signed_v
        .pointer("/proof/proof/data")
        .or_else(|| signed_v.pointer("/proof/proof"))
        .or_else(|| signed_v.pointer("/proof/data"))
        .and_then(Value::as_str)
        .ok_or("signedBlock missing proof bytes")?;
    let proof =
        MultiMessageAggregate::new(decode_hex_bytes(proof_hex)?).map_err(|e| e.to_string())?;
    let signed = SignedBlock::new(block, proof);
    verify_block_proof(&signed, &state.validators, &LeanMultisigVerifier).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "test_driver_tests.rs"]
mod tests;
