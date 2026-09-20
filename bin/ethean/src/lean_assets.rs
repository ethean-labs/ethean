//! Load Hive Lean assets (config.yaml + validators.yaml keys) for `ethean start`.

use ethean_node::{load_node_keys, load_validator_assignment, EtheanClient, LoadedNodeKeys};
use std::path::Path;
use tracing::{info, warn};

/// Log assignment and load privkey files when the registry path is set.
pub fn load_optional_registry(
    registry_path: Option<&str>,
    node_id: &str,
) -> Result<Option<LoadedNodeKeys>, Box<dyn std::error::Error + Send + Sync>> {
    let Some(path) = registry_path else {
        return Ok(None);
    };
    let path = Path::new(path);
    let assignment = load_validator_assignment(path, node_id)?;
    info!(
        node_id = %assignment.node_id,
        indices = ?assignment.indices,
        path = %path.display(),
        "loaded validator registry assignment"
    );
    match load_node_keys(path, node_id) {
        Ok(keys) => {
            info!(
                proposal = keys.proposal.is_some(),
                attestation = keys.attestation.is_some(),
                "loaded registry privkey files"
            );
            Ok(Some(keys))
        }
        Err(e) => {
            warn!(error = %e, "registry privkey files not loaded");
            Ok(None)
        }
    }
}

/// Apply loaded registry keys onto the client (best-effort).
pub fn apply_registry_keys(client: &mut EtheanClient, keys: Option<&LoadedNodeKeys>) {
    if let Some(keys) = keys {
        client.apply_registry_keys(keys);
    }
}
