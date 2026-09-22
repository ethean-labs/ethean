//! Install Hive registry keys onto a running client.

use crate::client::EtheanClient;
use crate::local_attester::LocalAttester;
use crate::local_proposer::LocalProposer;
use crate::registry_keys::LoadedNodeKeys;
use tracing::{info, warn};

impl EtheanClient {
    /// Prefer Hive registry proposal/attestation keys when present.
    ///
    /// Keys are native XMSS; a key that fails to install leaves the existing
    /// signer unchanged and logs a warning. Installing a proposer also starts
    /// the proof service when an `ethean-prover` binary is available.
    pub fn apply_registry_keys(&mut self, keys: &LoadedNodeKeys) {
        info!(
            node_id = %keys.node_id,
            indices = ?keys.indices,
            has_proposal = keys.proposal.is_some(),
            has_attestation = keys.attestation.is_some(),
            "applying Hive validator registry keys"
        );
        if !keys.indices.is_empty() {
            self.owner.owned_validator_indices = keys.indices.clone();
        }
        if let Some(att) = keys.attestation.clone() {
            match LocalAttester::from_key_record(att) {
                Ok(attester) => {
                    info!(
                        production = attester.is_production(),
                        "installed registry attestation privkey into LocalAttester"
                    );
                    self.owner.attester = Some(attester);
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "registry attestation privkey not installed"
                    );
                }
            }
        }
        let Some(prop) = keys.proposal.clone() else {
            return;
        };
        match LocalProposer::from_key_record(prop) {
            Ok(proposer) => {
                info!(
                    production = proposer.is_production(),
                    "installed registry proposal privkey into LocalProposer"
                );
                self.owner.proposer = Some(proposer);
                self.ensure_prover();
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "registry proposal privkey not installed; keeping existing local proposer"
                );
            }
        }
    }
}
