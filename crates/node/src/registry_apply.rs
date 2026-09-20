//! Install Hive registry keys onto a running client.

use crate::client::EtheanClient;
use crate::local_proposer::LocalProposer;
use crate::registry_keys::LoadedNodeKeys;
use tracing::{info, warn};

impl EtheanClient {
    /// Prefer Hive registry proposal/attestation keys when present.
    ///
    /// Proposal import requires `leansig-backend`; otherwise the existing local
    /// proposer (smoke / production keygen) is left unchanged and a warning is logged.
    pub fn apply_registry_keys(&mut self, keys: &LoadedNodeKeys) {
        info!(
            node_id = %keys.node_id,
            indices = ?keys.indices,
            has_proposal = keys.proposal.is_some(),
            has_attestation = keys.attestation.is_some(),
            "applying Hive validator registry keys"
        );
        if let Some(ref att) = keys.attestation {
            info!(
                index_key = ?att.key_id,
                "attestation registry key loaded (duty wiring still uses local attester path)"
            );
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
