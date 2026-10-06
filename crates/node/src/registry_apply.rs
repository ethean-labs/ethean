//! Install Hive registry keys onto a running client.

use crate::chain_owner::ValidatorSigners;
use crate::client::EtheanClient;
use crate::local_attester::LocalAttester;
use crate::local_proposer::LocalProposer;
use crate::registry_keys::{LoadedNodeKeys, ValidatorKeys};
use tracing::{info, warn};

impl EtheanClient {
    /// Prefer Hive registry proposal/attestation keys when present.
    ///
    /// Keys are native XMSS and installed per validator index; a key that fails
    /// to install is skipped with a warning. Installing a proposer also starts
    /// the proof service when an `ethean-prover` binary is available.
    pub fn apply_registry_keys(&mut self, keys: &LoadedNodeKeys) {
        info!(
            node_id = %keys.node_id,
            indices = ?keys.indices,
            proposal_keys = keys.proposal_count(),
            attestation_keys = keys.attestation_count(),
            "applying Hive validator registry keys"
        );
        if !keys.indices.is_empty() {
            self.owner.owned_validator_indices = keys.indices.clone();
        }
        let mut any_proposer = false;
        for validator in &keys.validators {
            let signers = self.install_validator(validator);
            if signers.attester.is_none() && signers.proposer.is_none() {
                continue;
            }
            any_proposer |= signers.proposer.is_some();
            self.owner.signers.insert(validator.index, signers);
        }
        if any_proposer {
            self.ensure_prover();
        }
    }

    fn install_validator(&mut self, validator: &ValidatorKeys) -> ValidatorSigners {
        let index = validator.index;
        let mut signers = ValidatorSigners::default();
        if let Some(att) = validator.attestation.clone() {
            let secret = att.secret.clone();
            match LocalAttester::from_key_record(att) {
                Ok(attester) => {
                    self.owner.key_prep.track(secret);
                    info!(
                        index,
                        production = attester.is_production(),
                        "installed registry attestation privkey"
                    );
                    signers.attester = Some(attester);
                }
                Err(e) => warn!(error = %e, index, "registry attestation privkey not installed"),
            }
        }
        if let Some(prop) = validator.proposal.clone() {
            let secret = prop.secret.clone();
            match LocalProposer::from_key_record(prop) {
                Ok(proposer) => {
                    self.owner.key_prep.track(secret);
                    info!(
                        index,
                        production = proposer.is_production(),
                        "installed registry proposal privkey"
                    );
                    signers.proposer = Some(proposer);
                }
                Err(e) => warn!(error = %e, index, "registry proposal privkey not installed"),
            }
        }
        signers
    }
}
