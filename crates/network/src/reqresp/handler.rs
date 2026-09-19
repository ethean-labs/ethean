//! Status exchange against local chain identity.

use ethean_network_wire::Status;

use crate::error::{NetworkError, Result};

/// Local and remote Status after a successful compatibility check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusExchange {
    pub local: Status,
    pub remote: Status,
}

/// Validate remote Status against local genesis/fork; keep head claims untrusted.
pub fn handle_status(local: &Status, remote: Status) -> Result<StatusExchange> {
    remote
        .compatible_with(local)
        .map_err(|e| NetworkError::Handshake(e.to_string()))?;
    Ok(StatusExchange {
        local: local.clone(),
        remote,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_fork_mismatch() {
        let local = Status {
            genesis_root: [1u8; 32],
            fork_segment: "aaaa1111".into(),
            head_slot: 0,
            head_root: [0u8; 32],
            finalized_slot: 0,
            finalized_root: [0u8; 32],
        };
        let mut remote = local.clone();
        remote.fork_segment = "bbbb2222".into();
        assert!(handle_status(&local, remote).is_err());
    }
}
